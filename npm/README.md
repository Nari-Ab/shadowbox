# shadowbox-cli

Zero-trust copy-on-write sandbox and interactive time-machine for AI coding agents (Claude Code, Cursor, Aider, OpenHands).

## Usage

Run immediately without prior installation:

```bash
npx shadowbox-cli claude
```

Or install globally:

```bash
npm install -g shadowbox-cli
shadowbox aider
```

## How It Works

1. Spawns an instant copy-on-write scratch copy of your workspace.
2. Sanitizes environment variables and cloaks `.env` and secrets.
3. Launches the AI coding agent inside the sandbox.
4. Shows interactive split-screen diffs of all file changes in real time.
5. Press `c` to commit changes atomically, or `d` to discard everything cleanly.

For source code, issues, and native Rust binaries:
https://github.com/Nari-Ab/shadowbox
