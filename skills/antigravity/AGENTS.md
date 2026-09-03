# cheat-engine-mcp Rules

- Automatically trigger the `cheat-engine-mcp` skill for any request related to operating, reverse-engineering, testing, installing, or troubleshooting the repo or its MCP tools.
- Windows is fully supported natively for memory scanning, process listing, memory reading, and memory writing (using Win32 API). Do not claim that memory scanning is Linux-only.
- Never write to memory or install hooks without calling preview tools first.
- Always use `confirm_write: true`, `max_writes` limits, and `dry_run: true` if applicable.
- GDB dynamic hooks require explicit `confirm_hook: true` or `confirm_probe: true` (GDB hooking is Linux-only).
- Keep reverse findings in `.cheat-tables/` and `reverse/` directories. Do not commit or expose these local directories.
