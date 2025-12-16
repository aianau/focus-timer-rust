# Focus Timer Rust (Dioxus) Implementation Plan

This plan outlines the steps to build the "focus-timer-rust" desktop application using Dioxus. The app will feature a modern dark UI, a circular countdown timer, and configurable work/break durations.

## 1. Project Setup

- [ ] Add dependencies to `Cargo.toml`: `dioxus` (features: ["desktop", "hooks"]), `dioxus-logger`.
- [ ] Create `src/main.rs` with a basic Dioxus desktop launch configuration.
- [ ] Create a `assets/` directory for styles (CSS) and potentially a background image placeholder.

## 2. Core Logic (State Management)

- [ ] Create `src/state.rs` to handle timer logic.
    - [ ] Define `TimerState` struct (current time, duration, is_running, mode: Work/Pause).
    - [ ] Implement `tick` function to decrement timer.
    - [ ] Implement `toggle_timer` and `reset_timer`.
    - [ ] Implement `set_duration` for editing.

## 3. UI Components

- [ ] **Main Layout**: Create the app shell in `src/main.rs` (or `src/app.rs`) with a dark background and centered content.
- [ ] **Circular Progress Component**:
    - [ ] Create `src/components/timer_circle.rs`.
    - [ ] Use SVG or CSS conic-gradient to render the circular progress bar based on `current_time / total_time`.
    - [ ] Display the digital time in the center.
- [ ] **Controls**:
    - [ ] Add Start/Pause toggle button.
    - [ ] Add mode switching (Work vs. Pause) controls (if not automatic, or just standard buttons as per screenshot).
- [ ] **Settings Modal**:
    - [ ] Create `src/components/settings.rs`.
    - [ ] Add input fields to edit "Work Duration" and "Pause Duration".
    - [ ] Logic to update the global timer state.

## 4. Styling

- [ ] Create `assets/style.css`.
- [ ] Implement the dark theme:
    - [ ] Background image/gradient overlay.
    - [ ] Typography (large clear numbers).
    - [ ] Glassmorphism effects (translucent panels) if matching the screenshot closely.

## 5. Integration

- [ ] Wire up the `use_interval` or standard Rust thread/channel to drive the timer ticks in the Dioxus component.
- [ ] Ensure the UI updates smoothly every second.