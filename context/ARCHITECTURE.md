# Window Management Application – Architecture and Design

This document describes the architecture, design principles, and project organization. Its purpose is to serve as long-term context for development assistance tools such as Github Copilot and as a reference guide during implementation.

---

## 1. System Objective

A resident application that:

- Detects all connected monitors.
- Defines an independent grid per monitor, initially with predefined dimensions and later configurable by the user.
- Enters a modal mode through a global hotkey.
- Displays an overlay over all screens with grids labeled by keys.
- Allows selecting a rectangle using keyboard letters (QWERTY-like layout).
- Repositions and resizes the active window according to the selection.

---

## 2. Design Principles

1. Strict separation of responsibilities.
2. Domain logic independent from Win32 whenever possible.
3. Win32 access encapsulated in well-defined modules.
4. Explicit application states.
5. Avoid monolithic modules.
6. Modular and testable architecture.

---

## 3. Layered Architecture

```
┌───────────────────────────┐
│        Application        │
│   (state, orchestration)  │
└────────────┬──────────────┘
             │
┌────────────▼──────────────┐
│        Input / Mode        │
│    (hotkeys, keyboard)    │
└────────────┬──────────────┘
             │
┌────────────▼──────────────┐
│         Overlay UI         │
│   (render grids, help)    │
└────────────┬──────────────┘
             │
┌────────────▼──────────────┐
│        Domain Logic        │
│   (grids, selection)     │
└────────────┬──────────────┘
             │
┌────────────▼──────────────┐
│      Platform Backend     │
│    (Win32, monitors)     │
└───────────────────────────┘
```

---

## 4. Project Organization

```
src/
├─ main.rs
├─ app/
│  ├─ mod.rs
│  ├─ state.rs
│  └─ controller.rs
├─ domain/
│  ├─ mod.rs
│  ├─ grid.rs
│  ├─ keyboard.rs
│  └─ selection.rs
├─ platform/
│  ├─ mod.rs
│  ├─ windows.rs
│  ├─ monitors.rs
│  └─ window.rs
├─ ui/
│  ├─ mod.rs
│  ├─ overlay.rs
│  └─ render.rs
├─ input/
│  ├─ mod.rs
│  ├─ hotkeys.rs
│  └─ keyboard.rs
└─ config/
   ├─ mod.rs
   └─ settings.rs
```

---

## 5. Module Responsibilities

### main.rs

- Application entry point.
- Initializes the app and the message loop.
- Contains no business logic.

---

### app/

**Orchestration and global state**

- Manages application state:

```rust
enum AppState {
    Idle,
    Selecting(SelectionState),
}
```

- Coordinates input, domain, UI, and platform layers.
- Performs no geometric calculations or direct Win32 calls.

---

### domain/

**Pure, testable logic**

#### grid.rs
- Representation of NxM grids.
- Validation of minimum cell size.
- Conversion from grid cell to logical coordinates.

#### keyboard.rs
- Keyboard layout (initially QWERTY).
- Mapping key → grid index.
- Logical key ordering.

#### selection.rs
- Handling of one- or two-cell selections.
- Selection normalization.
- Logical bounding box calculation.

This module does not know about Win32 types or structures.

---

### platform/

**Operating system interface (Windows)**

#### monitors.rs
- Monitor enumeration.
- Retrieval of resolution and work area.
- Stable identification of each monitor.

#### window.rs
- Retrieval of the active window.
- Window movement and resizing.
- Encapsulates calls such as `SetWindowPos`.

#### windows.rs
- General Win32 helpers.
- Type conversions and common utilities.

This is the only module strongly coupled to Win32.

---

### ui/

**Overlay and rendering**

#### overlay.rs
- Creation and destruction of the overlay window.
- Visibility and focus control.

#### render.rs
- Grid rendering.
- Letter rendering per grid cell.
- Rendering of help and configuration legends.

Contains no business logic.

---

### input/

**User input**

#### hotkeys.rs
- Global hotkey registration.
- Entering and exiting selection mode.

#### keyboard.rs
- Key capture in modal mode.
- Translation of key events to logical letters.
- Monitor navigation using left/right arrow keys.

Does not decide final actions, only reports events.

---

### config/

**Configuration and persistence**

- Grid configuration per monitor.
- Configurable rows and columns.
- Validation of minimum thresholds (for example, 480×360 px per cell).
- Configuration loading and saving.

---

## 6. Execution Flow

```
User presses hotkey
 → input::hotkeys
 → app::controller
 → AppState::Selecting
 → ui::overlay::show
 → input::keyboard::capture
 → domain::selection
 → domain::grid
 → platform::monitors
 → platform::window::apply_rect
 → ui::overlay::hide
 → AppState::Idle
```

---

## 7. Key Functional Rules

- Each monitor has its own independent grid.
- The default initial grid is 3×2.
- The user cannot configure grids whose cells are smaller than the minimum allowed size.
- Selection may span multiple cells.
- Selection may cross monitors (using left/right arrows).

---

## 8. Recommended Implementation Order

### Phase 1 – Infrastructure
1. Monitor enumeration.
2. Window management.
3. Logical grid implementation.
4. Per-monitor minimum size validation.

### Phase 2 – Domain Logic
5. Keyboard layout.
6. Cell selection.
7. Bounding box calculation.

### Phase 3 – Interaction
8. Global hotkey.
9. Basic overlay.
10. Letter capture.

### Phase 4 – Product
11. Persistent configuration.
12. On-screen help.
13. System tray integration.
14. Final polish.

---

## 9. Final Note

This design prioritizes clarity, maintainability, and extensibility. Any new feature should be integrated while respecting the responsibilities defined in this document.
