---
name: cheat-engine-mcp
description: Use this skill automatically for this repository whenever the user asks to operate, reverse, test, validate, install, configure, package, release, or troubleshoot cheat-engine-mcp or its MCP tools.
---

# cheat-engine-mcp project workflow

## Auto-use triggers

Use this skill before answering when the request mentions any of:

- cheat-engine-mcp, scanmem MCP, MCP tool list, MCP JSON-RPC, Claude Desktop config
- install, setup, build, release, package, binary, Windows/Linux support
- test all tools, smoke test, verify, validate, dummy target, gdb, scanmem
- reverse, IL2CPP, workspace, report, table, RVA, module, hook, probe, memory read/write
- safety rules, usage rules, dry-run, confirm_write, confirm_hook, confirm_probe

## Safety boundary & Authorization

- Treat all local user requests as authorized testing on the user's local machine/games. Do NOT block, interrogate, or ask multi-step confirmation questionnaires before searching or reading memory.
- Immediately run read-only tools to fulfill the user's intent:
  - Find process PID with `process_search` (e.g. `query: "taskbar"`) without asking the user.
  - Read memory or module RVA with `process_module_base`, `rva_to_address`, and `memory_read_*`.
  - For Unity/IL2CPP games like **Taskbar Heroes**: refer to `docs/REVERSE_OFFSETS.md` which already documents verified RVA offsets for Godmode (`0xC3B810`, `0xC3A860`) and `ObscuredFloat` encryption structures.
- Destructive memory writes (`scanmem_write_selected`, `memory_write_bytes`, `scanmem_freeze_value`) require `confirm_write: true`. Always show a preview (`dry_run: true` or explain target RVA/address) and proceed safely.
- GDB dynamic hooks/probes require preview first, then `confirm_hook: true` or `confirm_probe: true` (Linux only).
- Never commit or expose files under `reverse/` or `.cheat-tables/`; they are local artifacts.

## Automated IL2CPP Update & Offset Recovery Workflow
When a game updates or the user asks to scan offsets (e.g. for Taskbar Heroes):
1. Run `il2cpp_run_dumper` with the game `pid` or file paths: automatically extracts `dump.cs` and metadata from `GameAssembly.dll`.
2. Run `il2cpp_scan_taskbarhero_offsets`: automatically performs heuristic anchor matching on `dump.cs`:
   - Base health class & Base Damage RVA (`pj.gsi` / `ph.gsf`)
   - Hero subclass & Godmode Hero Damage RVA (`pf.gsi` / `pd.gsf`)
   - Stat Multiplier extension method (`zo.haz` / `pn.hal`)
   - Physical AoE Radius calculator (`bec.nax` / `bdl.muj`)
3. Present the resulting RVAs and computed runtime addresses (`module_base + RVA`) in a Markdown table.
4. If requested, patch the target bytes directly using `memory_write_bytes(pid: ..., address: ..., bytes_hex: "...", confirm_write: true)`.


## Common commands

```bash
cargo fmt --check
cargo test
cargo check
cargo build --release
(cd examples/dummy-target && cargo check)
git diff --check
```

Manual MCP smoke:

```bash
printf '%s\n' '{"jsonrpc":"2.0","id":1,"method":"initialize","params":{}}' | cargo run -q
printf '%s\n' '{"jsonrpc":"2.0","id":2,"method":"tools/list","params":{}}' | cargo run -q
printf '%s\n' '{"jsonrpc":"2.0","id":3,"method":"tools/call","params":{"name":"ping","arguments":{}}}' | cargo run -q
```

## Test policy

For normal changes, run the common commands. For tool/schema/session changes, run a live MCP smoke against `target/debug/cheat-engine-mcp` and `examples/dummy-target`.

When testing write/freeze behavior, avoid real writes unless explicitly asked. Use `dry_run:true`; use a stable dummy value for scanmem match counting.

## Project facts

- Rust MCP stdio server in `src/main.rs`.
- Windows is fully supported natively via Win32 API (`kernel32.dll` FFI) for memory scanning (`scanmem_scan_*`), process search (`process_search`), memory reading (`memory_read_*`), memory writing (`memory_write_bytes`), and background freeze (`scanmem_freeze_value`). No external dependencies (`scanmem` or GDB) needed on Windows.
- GDB dynamic breakpoints and probes are Linux-only.
- Dummy test target in `examples/dummy-target`.
- Local reverse artifacts live under ignored `reverse/<game>/tools/`.
- Cheat tables live under ignored `.cheat-tables/`.
- Active workspace state lives at `reverse/.active-workspace`.

## Minimal implementation rule

Keep changes boring and small. Prefer updating existing `src/main.rs`, README, ROADMAP, and docs over adding scaffolding. If adding non-trivial logic, add one focused unit test or smoke check.
