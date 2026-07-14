# AGENTS.md

Workspace instructions for ZCode agents working in this repository.

## What this is

SpearX — a cross-platform desktop **tool manager** for developers/security researchers. Lets users organize and one-click-launch external tools (Java JARs, terminal tools, web apps, binaries). Migrated from Wails v3 (Go) to **Tauri v2 (Rust)**; see `docs/TAURI_MIGRATION_PLAN.md`. The old Go `app.go` is referenced in comments as the behavioral reference ("对齐 Go ...") — match its semantics when in doubt.

Primary (and currently first-class) platform is **macOS**; Windows is built and shipped; Linux is stubbed.

## Architecture & layout

Two-process app: **Rust backend** (`src-tauri/`) + **Vue 3 frontend** (`frontend/`), bridged by Tauri IPC (`invoke`).

```
src-tauri/src/
  main.rs          # tauri::Builder entry: plugin registration, all command handlers, macOS/Windows vibrancy setup
  lib.rs           # module declarations (spearx_lib)
  models.rs        # data structs — see "Dual data model" below
  config.rs        # read/write tool.yml, atomic save w/ backup, path cleaning
  executor.rs      # tool launch engine (Java/Open/openterm/Browser/Binary)
  scanner.rs       # directory scan + tool-type auto-detection
  notes.rs         # per-tool Markdown notes
  paths.rs         # platform config/resource dirs
  maintenance.rs   # startup config repair + path cleanup (spawned at launch)
  platform/        # mod.rs (cross-platform fns) + macos.rs / windows.rs / linux.rs via cfg
  commands/        # #[tauri::command] modules, grouped by domain (tools, categories, execution, ...)
frontend/src/
  App.vue          # the ENTIRE UI — single ~6200-line file, no router, no components/, no store
  api/index.ts     # typed wrappers over every Tauri command (source of truth for IPC names)
  main.js          # boots Vue + Element Plus + all icons
  styles/main.css  # global styles incl. frosted-glass CSS
```

**Layer rule:** frontend never touches the filesystem directly. It calls `api/index.ts` → Tauri `invoke` → a `#[tauri::command]` in `commands/` → logic modules (`config`, `executor`, …). Platform-specific behavior lives behind `cfg` in `platform/` and `paths.rs`, not in command modules.

## Common commands

```bash
# Dev (hot reload). Runs Vite on http://localhost:9245 (strictPort) + Rust rebuild.
cargo tauri dev

# Production build (.app/.dmg on macOS, .msi/.exe on Windows)
cargo tauri build

# Rust tests (config parsing, model conversions). Note: integration tests read the real
# user config at ~/Library/Application Support/SpearX/tool.yml when present.
cd src-tauri && cargo test

# Frontend only
cd frontend && npm install && npm run dev   # or: npm run build
```

No lint/typecheck scripts are configured for the frontend (no `tsc`, no eslint). `frontend/package.json` name is `sspsec-tools` (legacy) — do not "fix" it.

## Critical conventions

### Dual data model (models.rs)
The same data has **two serde representations** — adding/changing a field means touching both:
- **YAML structs** (`ConfigYaml`, `CategoryYaml`, `ToolYaml`) — `PascalCase`/specific keys matching `tool.yml` exactly (`ToolName`, `PATH`, `VALUE`, `FileName`, …). Used for on-disk config.
- **JSON structs** (`Tool`, `Category`, `Categories`, `ScannedTool`, `FileInfo`) — `#[serde(rename_all = "camelCase")]`. Used for IPC with the frontend.
- `From`/`Into` impls convert between them; keep these in sync. The TypeScript interfaces in `api/index.ts` must mirror the JSON structs.

Exception: `JavaConfig` is `PascalCase` for *both* YAML and JSON (frontend reads `config.Java8/Java11/Java17`) — do not camelCase it.

### Adding a Tauri command
1. Write the `#[tauri::command] fn` in the right `commands/*.rs`.
2. Register it in `src-tauri/src/main.rs` `tauri::generate_handler![...]` (handlers are grouped by domain with comments).
3. Add a typed wrapper in `frontend/src/api/index.ts`. IPC arg names are camelCase on the JS side but map to snake_case Rust params automatically.

### Permissions
Every IPC capability used by the frontend must be granted in `src-tauri/capabilities/main.json`. Adding a new shell/fs/opener operation usually needs a new `shell:*` / `fs:*` permission entry or it silently fails at runtime.

### Config I/O safety
All writes go through `config::save_categories_to_file`, which: backs up → `.tmp` write → atomic rename → re-parse validation → restore-from-backup on failure. Do not add direct `fs::write` to the config path. Config lives in the **user data dir** (survives app updates), not the bundle:
- macOS `~/Library/Application Support/SpearX/tool.yml`
- Windows `%APPDATA%/SpearX/tool.yml`
- Linux `~/.config/spearx/tool.yml`

### Platform-specific gotchas
- **macOS vibrancy** (`main.rs`, `#[cfg(target_os="macos")]`): applies `NSVisualEffectView` UnderWindowBackground, forces `NSAppearanceNameVibrantDark`, clears window background so WebView is fully transparent. This is deliberate — resizing flashes are avoided because the same layer shows during and after resize. Don't make the WebView opaque.
- **Window/vibrancy crate pinning**: `window-vibrancy`, `objc2`, `objc2-app-kit` versions are deliberately aligned with the `tao` dependency tree to avoid pulling a second copy. Bump them together with care.
- **Windows** uses Mica (`apply_mica`) via the same transparent-WebView approach; `spawn_hidden` sets `CREATE_NO_WINDOW` to suppress child consoles.
- **Linux `open_terminal` returns "不支持的平台"** — it's a stub. Don't assume Linux launch support works.
- Terminal launch: macOS prefers iTerm.app, falls back to Terminal.app via osascript; Windows uses `start cmd /K`. Paths are shell-escaped.

## Logging
Plain `println!` / `eprintln!` throughout (Chinese-language messages). The `log` crate is a declared dependency but its macros are not actually used — don't assume a logger is initialized.

## CI / release
`.github/workflows/build.yml` builds three targets on push/PR and on `v*` tags:
- macOS aarch64-apple-darwin (native)
- macOS x86_64-apple-darwin (cross-compiled on the same macos-latest runner)
- Windows x64

Pushing a `v*` tag creates a draft GitHub Release. `beforeBuildCommand`/`beforeDevCommand` run `npm run build`/`dev` in `../frontend`, so the frontend must be `npm install`-able before any Tauri build.

## Notes for agents
- `frontend/src/App.vue` is a monolith. Search within it rather than expecting a component tree.
- Comments and user-facing strings are largely in **Chinese**. Match the surrounding language.
- `.zcode/` is gitignored (scratch/tooling dir); don't commit it.
- Tool `VALUE` field is the dispatch key in `executor.rs`: `Java8`/`Java11`/`Java17`/`Open`/`openterm`/`Browser`/`Binary`. New launch types must be handled here.
