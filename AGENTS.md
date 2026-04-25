# Focus Timer (Rust) — Project Guide for Agents

## Overview

A modern, desktop-based focus timer application built with **Rust** and **Dioxus**. It helps manage time using the Pomodoro technique, tracking focus sessions and breaks with a clean, unobtrusive interface. Supports **Windows** and **macOS**.

---

## Quick Start

```bash
# Run in development mode
cargo run

# Build release
cargo build --release

# Run tests
cargo test
```

---

## Architecture

### Tech Stack

| Layer | Technology |
|-------|-----------|
| Language | Rust (Edition 2021) |
| UI Framework | [Dioxus](https://dioxuslabs.com/) (Desktop) |
| Async Runtime | Tokio |
| Notifications | `notify-rust` (cross-platform), `tauri-winrt-notification` (Windows toast) |
| System Tray | `tray-icon` |
| Window Management | `tao` |
| Config/State Persistence | `serde` + `serde_json` (JSON files) |
| SVG Rendering | `usvg` + `resvg` + `tiny-skia` |

### Module Structure

```
src/
├── main.rs          # Application entry point, Dioxus app root, timer/event loops
├── state.rs         # Core state management: TimerState, AppConfig, SessionHistory, Task, enums
├── tray.rs          # System tray icon creation, icon loading (SVG → RGBA)
├── events.rs        # AppEvent enum (Tray / Menu events bridged from sync → async)
└── components/
    ├── mod.rs       # Module re-exports
    ├── timer_circle.rs   # Circular progress SVG timer display
    ├── settings.rs       # Settings modal (duration, notifications, startup)
    ├── task_list.rs      # Today's task list (add/complete/remove)
    ├── titlebar.rs       # Custom undecorated title bar (drag, minimize, close)
    └── resize_handles.rs # 8-point custom window resize handles
```

### Data Flow

1. **State** (`TimerState`) is the single source of truth, held as a `use_signal` in `app()`.
2. **Timer tick** runs in a `tokio::select!` loop (`main.rs`), decrementing `current_time` every second.
3. **When timer reaches zero**, a session is complete and the app enters **overtime** mode (counting up). A notification is triggered based on user settings.
4. **User actions** (Start/Pause/Reset/Settings) mutate `TimerState` via `use_signal` writes, triggering Dioxus re-renders.
5. **Config** (`AppConfig`) and **history** (`SessionHistory`) are persisted to JSON files in the app directory on every change.

### Key Files

| File | Responsibility |
|------|---------------|
| `src/main.rs` | App launch, `app()` root component, timer loop, tray event bridge, notification dispatch |
| `src/state.rs` | `TimerState` struct with all timer logic (`tick`, `toggle`, `finish_current_session`, `switch_mode`), `AppConfig` load/save, `SessionHistory` (sessions, tasks, auto-delete), platform-specific startup/registry logic |
| `src/tray.rs` | `create_tray_icon()` with Exit menu, `load_icon_data()` (SVG → RGBA via usvg/resvg) |
| `src/events.rs` | `AppEvent` enum bridging sync tray-icon channels to async Dioxus |
| `src/components/timer_circle.rs` | SVG circular progress indicator, time display, overtime styling |
| `src/components/settings.rs` | Modal for work/pause duration, notification mode, hide completed, auto-delete, startup, start menu icon |
| `src/components/task_list.rs` | Task input, task list rendering with checkboxes and remove buttons |
| `src/components/titlebar.rs` | Custom title bar with drag region and minimize/close buttons |
| `src/components/resize_handles.rs` | Invisible divs at window edges for custom resize |
| `build.rs` | Copies `assets/` to build output directory; on Windows, converts SVG → ICO and embeds as window icon |
| `assets/style.css` | Dark theme CSS (CSS variables, custom components, modal, task list, resize handles) |
| `assets/timer-svgrepo-com.svg` | Source SVG for the timer icon |

### State Model (`state.rs`)

```
TimerState
├── current_time: Duration    # Time remaining (or overtime when at 0)
├── work_duration: Duration   # Configured work session length
├── pause_duration: Duration  # Configured break length
├── is_running: bool          # Timer is actively counting
├── mode: TimerMode           # Work | Pause
├── notification_mode: NotificationMode  # Popup | Notification | NotificationPersistent
├── history: SessionHistory   # All sessions + tasks
├── hide_completed_tasks: bool
├── auto_delete_old_tasks: bool
├── run_at_startup: bool
├── show_start_menu_icon: bool
├── window_width / window_height: u32
└── overtime: Duration        # Seconds past timer zero (counting up)

SessionHistory
├── sessions: Vec<FocusSession>  # { start_time, duration_secs, mode }
└── tasks: Vec<Task>             # { id, title, completed, created_at }

AppConfig          # Persisted to focus_timer_config.json
├── work_minutes: u64
├── pause_minutes: u64
├── notification_mode: Option<NotificationMode>
├── hide_completed_tasks: Option<bool>
├── auto_delete_old_tasks: Option<bool>
├── run_at_startup: Option<bool>
├── show_start_menu_icon: Option<bool>
└── window_width / window_height: Option<u32>
```

---

## Conventions & Patterns

### Dioxus Desktop

- **No window decorations**: `with_decorations(false)` — all chrome is custom (titlebar + resize handles).
- **`use_signal`** for all reactive state. `TimerState` is owned by the `app()` root component.
- **`use_future`** for long-running async tasks: timer loop, tray event bridge, window resize observer.
- **`rsx!`** macro for all JSX-like templating.
- **`include_bytes!`** in `tray.rs` embeds the SVG at compile time.

### Timer Logic

- Timer ticks every 1 second via `tokio::time::sleep`.
- When `current_time` reaches `Duration::ZERO`, the timer **does not stop** — it enters **overtime** mode (`overtime` counts up).
- User must interact (click Start/Pause button) to finish overtime and switch sessions.
- `tick()` returns `bool` — `true` when session just completed (reached 0).

### Notifications

| Platform | Mode | Behavior |
|----------|------|----------|
| Windows | `Popup` | Focuses the app window |
| Windows | `Notification` | Short-lived toast (no action buttons) |
| Windows | `NotificationPersistent` | Persistent toast with "Ok" and "Start" buttons; blocks a thread until dismissed or action |
| macOS | Any | Native system notification (no action buttons currently) |
| Linux | Any | Native system notification (no action buttons currently) |

Windows persistent notifications use a blocking thread with a 4-hour timeout to avoid leaks.

### Platform-Specific Code

- **Windows**: `#[cfg(target_os = "windows")]` — Winreg for startup registry, `tauri-winrt-notification` for toast, SVG→ICO embedding in `build.rs`.
- **macOS**: `#[cfg(target_os = "macos")]` — LaunchAgent plist for startup.
- **Linux**: Minimal support; notifications via `notify-rust`; no startup integration.

### Configuration Persistence

- `AppConfig` is saved to `focus_timer_config.json` in the **app directory** (same folder as the executable).
- `SessionHistory` is saved to `focus_history.json` in the same directory.
- Both files are auto-created on first use with default values.

### Assets

- The `assets/` directory **must** be alongside the executable at runtime (CSS + SVG icon).
- `build.rs` copies `assets/` to the target build directory during compilation.
- On Windows, `build.rs` also converts `timer-svgrepo-com.svg` → `icon.ico` and embeds it as a Windows resource.

---

## Build & Release

### Prerequisites

- Rust toolchain (stable)
- `cargo`, `rustc`

### Development

```bash
cargo run
```

### Release Build

```bash
cargo build --release
```

Output:
- **Windows**: `target/release/focus-timer-rust.exe`
- **macOS**: `target/release/focus-timer-rust`

### Manual Packaging

Copy executable + `assets/` directory:

```
MyTimerApp/
├── focus-timer-rust.exe   (or focus-timer-rust)
└── assets/
    ├── style.css
    └── timer-svgrepo-com.svg
```

### CI/CD (GitHub Actions)

- **Trigger**: Push of a `v*` tag (e.g., `v0.1.0`).
- **Jobs**:
  - `build-windows`: Builds on `windows-latest`, packages `focus-timer-rust-windows.zip`.
  - `build-macos`: Builds on `macos-latest`, creates a `.app` bundle → `focus-timer-rust-macos.zip`.
- Both jobs run `cargo test` and publish artifacts via `softprops/action-gh-release`.

---

## Testing

```bash
cargo test
```

Tests in `state.rs` cover:
- Timer initialization
- Timer tick (second decrement)
- Timer completion and overtime transition
- Mode switching
- Task CRUD operations

> **Note**: Tests use alternate file names (`test_focus_history.json`, `test_focus_timer_config.json`) to avoid polluting real user data.

---

## Key Implementation Details

### Window Resize Persistence
A `use_future` loop checks window size every 2 seconds and saves to config only when it changes.

### Tray Event Bridge
Tray-icon and menu events come from sync crossbeam channels. A spawned thread polls both and forwards to an async `tokio::sync::mpsc` channel for consumption in `use_future`.

### SVG Icon Pipeline
1. `usvg` parses the SVG file.
2. `tiny_skia` renders it to a `Pixmap` (RGBA).
3. `image` crate saves as `.ico` (Windows build).
4. `embed-resource` compiles the `.rc` file into the binary.

### Task Auto-Delete
When `auto_delete_old_tasks` is enabled, completed tasks older than 48 hours are removed on startup.

### Overtime Behavior
After the timer hits 00:00, the circular progress stays full and the time display switches to a `+MM:SS` format in a yellow color. The timer keeps counting until the user clicks the button.
