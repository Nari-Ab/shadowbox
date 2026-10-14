use clap::Parser;

#[derive(Parser, Debug)]
#[command(name = "shadowbox", version, about = "Zero-trust sandbox and time-machine for AI coding agents")]
struct Cli {
    /// Command and arguments to execute inside the sandbox
    #[arg(trailing_var_arg = true, required = true)]
    command: Vec<String>,
}

fn main() {
    let cli = Cli::parse();
    println!("ShadowBox v{} starting: {:?}", shadowbox::version(), cli.command);
}
