# claude-stream-format

A fast Rust CLI tool that formats Claude Code's `--output-format stream-json` output into human-readable text.

## Installation

```bash
cargo install --path .
```

Or build from source:

```bash
cargo build --release
```

## Usage

Pipe Claude Code's stream-json output through this tool:

```bash
claude -p "do something" --output-format stream-json | claude-stream-format
```

## Output Format

The tool formats different message types with icons:

| Tool | Format |
|------|--------|
| Read | 📖 Read: `<file_path>` |
| Edit | ✏️ Edit: `<file_path>` |
| Write | 📝 Write: `<file_path>` |
| Bash | 💻 Bash: `<command>` (truncated to 80 chars) |
| Glob | 🔍 Glob: `<pattern>` |
| Grep | 🔍 Grep: `<pattern>` |
| TodoWrite | 📋 TodoWrite |
| Task | 🤖 Task: `<description>` |
| Other | 🔧 `<tool_name>` |

Results are shown as: ✅ Done: `<result>`

## License

MIT License - see [LICENSE](LICENSE) for details.
