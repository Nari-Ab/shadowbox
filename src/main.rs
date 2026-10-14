use anyhow::{Context, Result};
use clap::{Parser, Subcommand};
use crossterm::{
    event::{self, Event},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{backend::CrosstermBackend, Terminal};
use shadowbox::runner::PtyRunner;
use shadowbox::session::Session;
use shadowbox::shield::SecretShield;
use shadowbox::tui::app::{AppAction, TuiApp};
use shadowbox::tui::ui;
use std::collections::HashMap;
use std::env;
use std::fs;
use std::io::stdout;
use std::path::PathBuf;
use std::time::{Duration, Instant};

#[derive(Parser, Debug)]
#[command(
    name = "shadowbox",
    author = "ShadowBox Contributors",
    version,
    about = "Zero-trust sandbox and time-machine for AI coding agents",
    long_about = "ShadowBox wraps autonomous AI agents in a zero-copy CoW sandbox with secret protection, live terminal streaming, and one-click rollback."
)]
struct Cli {
    #[command(subcommand)]
    subcommand: Option<Commands>,

    /// Run headless in plain terminal without interactive Ratatui TUI
    #[arg(long)]
    no_tui: bool,

    /// Disables SecretShield (passes all raw environment variables and .env untouched)
    #[arg(long)]
    allow_secrets: bool,

    /// Command and arguments to execute inside the sandbox
    #[arg(trailing_var_arg = true)]
    command: Vec<String>,
}

#[derive(Subcommand, Debug)]
enum Commands {
    /// Cleans orphaned sessions and scratch directories
    Clean {
        /// Target directory to clean (default: current directory)
        #[arg(short, long)]
        path: Option<PathBuf>,
    },
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    // Handle subcommands
    if let Some(Commands::Clean { path }) = cli.subcommand {
        let root = path.unwrap_or_else(|| env::current_dir().unwrap_or_default());
        let sessions_dir = root.join(".shadowbox");
        if sessions_dir.exists() {
            fs::remove_dir_all(&sessions_dir)
                .context("Failed to clean .shadowbox directory")?;
            println!("cleaned .shadowbox sessions in {}", root.display());
        } else {
            println!("no .shadowbox sessions found in {}", root.display());
        }
        return Ok(());
    }

    if cli.command.is_empty() {
        println!("error: no command specified.");
        println!("usage: shadowbox <command...>");
        println!("example: shadowbox claude --dangerously-skip-permissions");
        println!("try 'shadowbox --help' for options.");
        std::process::exit(1);
    }

    let cwd = env::current_dir().context("Failed to get current directory")?;

    // 1. Initialize Session
    let session = Session::new(&cwd, None).context("Failed to initialize shadowbox session")?;

    // 2. Prepare Environment & Shield
    let mut raw_env = HashMap::new();
    for (k, v) in env::vars() {
        raw_env.insert(k, v);
    }

    let (sanitized_env, blocked_count) = if cli.allow_secrets {
        let count = 0;
        (raw_env, count)
    } else {
        let sanitized = SecretShield::sanitize_env(&raw_env);
        let blocked = raw_env.len().saturating_sub(sanitized.len());
        (sanitized, blocked)
    };

    let cloaked_count = if !cli.allow_secrets {
        SecretShield::cloak_sensitive_files(session.work_dir())
            .unwrap_or_default()
            .len()
    } else {
        0
    };

    // 3. Spawn child process in PTY
    let mut pty_handle = PtyRunner::spawn(&cli.command, session.work_dir(), &sanitized_env)
        .context("Failed to spawn process in PTY")?;

    if cli.no_tui {
        // Headless execution mode
        println!("shadowbox: sandbox active at {}", session.work_dir().display());
        println!("executing: {:?}\n", cli.command);

        let output = pty_handle.read_all_output()?;
        let status = pty_handle.wait()?;
        print!("{}", String::from_utf8_lossy(&output));

        println!("\n--------------------------------------------------");
        if status.success() {
            println!("process exited cleanly.");
            println!("applying changes to host...");
            session.commit()?;
            println!("host workspace updated.");
        } else {
            println!("process failed with status: {:?}", status);
            println!("discarding sandbox changes.");
            session.discard()?;
        }
        return Ok(());
    }

    // 4. Interactive Ratatui TUI mode
    enable_raw_mode()?;
    let mut stdout = stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let mut app = TuiApp::new(session, cli.command.clone());
    app.blocked_vars_count = blocked_count;
    app.cloaked_secrets_count = cloaked_count;

    let mut last_diff_check = Instant::now();
    let res = run_tui_loop(&mut terminal, &mut app, &mut pty_handle, &mut last_diff_check);

    // Restore terminal before exit
    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    terminal.show_cursor()?;

    if let Err(err) = res {
        eprintln!("Error during execution: {:?}", err);
    }

    Ok(())
}

fn run_tui_loop<B: ratatui::backend::Backend>(
    terminal: &mut Terminal<B>,
    app: &mut TuiApp,
    pty_handle: &mut shadowbox::runner::PtyHandle,
    last_diff_check: &mut Instant,
) -> Result<()> {
    loop {
        // Read available output from PTY
        if let Ok(bytes) = pty_handle.read_available() {
            if !bytes.is_empty() {
                app.append_terminal_output(&bytes);
            }
        }

        // Check if child has exited
        if app.is_running {
            if let Ok(Some(status)) = pty_handle.try_wait() {
                app.is_running = false;
                app.exit_code = Some(status.exit_code() as i32);
                let _ = app.refresh_diffs();
            }
        }

        // Refresh file diffs every 500ms
        if last_diff_check.elapsed() > Duration::from_millis(500) {
            let _ = app.refresh_diffs();
            *last_diff_check = Instant::now();
        }

        // Render UI
        terminal.draw(|frame| ui::render(app, frame))?;

        // Handle user input
        if event::poll(Duration::from_millis(50))? {
            if let Event::Key(key) = event::read()? {
                match app.handle_key(key) {
                    AppAction::CommitAndExit => {
                        let _ = pty_handle.kill();
                        app.session.commit()?;
                        break;
                    }
                    AppAction::DiscardAndExit => {
                        let _ = pty_handle.kill();
                        app.session.discard()?;
                        break;
                    }
                    _ => {}
                }
            }
        }
    }

    Ok(())
}
