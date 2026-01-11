# Phase 4: Single-Monitor MVP & Configuration

> NOTE FOR AI AGENTS AND EDITORS: Any ~~struck-through~~ text in this project’s documentation represents outdated guidance. Follow sections explicitly marked with [ADDED] and the new Phase 4/5 split described here.

~~Previous title: Phase 4A: Advanced Core Features (multi-monitor focused)~~

**Status**: Phases 1–3 Complete ✓  
**Prerequisite**: Working single-monitor prototype with keyboard capture, overlay, and basic positioning  
**Goal**: [ADDED] Deliver a production-ready MVP for **single-monitor setups only**, including rotation-aware grids, optional snapping, and basic configuration UI.

---

## [ADDED] Phase 4 Overview

Phase 4 consolidates everything built in Phases 1–3 into a **stable, shippable MVP for single-monitor users only**. Multi-monitor support and broader polish are explicitly deferred to Phase 5.

### [ADDED] Scope Boundary

- In scope: **one physical monitor** (the primary or currently active one).
- Out of scope: Any cross-monitor movement, spanning, or per-monitor configuration.
- Objective: When this phase is done, a single-monitor user can rely on tactile-win for daily use.

### [ADDED] Core Objectives

1. **Single-Monitor Robustness**  
   Ensure that all existing features (overlay, selection, window positioning) behave correctly and predictably for users with exactly one active monitor.

2. **Rotation-Aware Grids**  
   Detect monitor orientation (landscape vs. portrait) and adapt:
   - Grid layout (rows/columns) to maintain useful cell shapes.
   - Minimum cell-size constraints according to the new orientation.

3. **Snapping-to-Grid Behavior**  
   Implement optional snapping logic so that when a rectangle lies adjacent to the monitor border, the final window rectangle:
   - Snaps cleanly to the relevant screen edge(s).
   - Respects a configurable **gap size** between windows and screen borders.
   - Can be **fully disabled** for users who prefer exact grid-based sizing.

4. **Basic Single-Monitor Configuration UI**  
   Provide a minimal but robust way for users to configure the single-monitor setup, either:
   - Inside the main app, or
   - In a small companion configuration app that manages startup and settings.

---

## [ADDED] Functional Requirements

### Single-Monitor Only Guarantee

- Application must **refuse or gracefully degrade** when multiple monitors are detected (e.g., show a clear message or operate only on the primary monitor without ambiguity).
- All internal APIs that depend on monitor information should have an explicit **single-monitor code path** (no `Vec<Monitor>` assumptions in Phase 4 logic).

### Rotation Handling

- Use the existing `platform::monitors` layer to:
  - Detect current orientation (width vs. height) for the active monitor.
  - React to orientation changes (e.g., system hotplug or rotation events) by recomputing the grid.
- Grid behavior:
  - Landscape: prefer default 3×2 or 4×2 layouts as defined in Phase 2.
  - Portrait: automatically switch to layouts better suited for tall monitors (e.g., 2×3, 2×4), subject to minimum cell-size constraints.
  - Enforce minimum cell sizes in **both** orientations; reject invalid combinations in the UI.

### Snapping Behavior and Gaps

- Domain additions (likely in `domain::core` / `domain::grid`):
  - Utility functions to:
    - Detect when a selection rectangle touches or is within a small epsilon of the monitor edges.
    - Expand or contract the rectangle to snap to those edges.
    - Apply a **gap inset** so the final window rectangle is slightly smaller than the cell area.
- Configuration:
  - Boolean flag: `snapping_enabled` (default: on for MVP, but easily tweakable).
  - Numeric `gap_px` (global gap size in pixels, small default like 8–12 px).
- Behavior:
  - If `snapping_enabled` is false → behave exactly as in Phase 3 (pure grid-based rectangle).
  - If true → snap to edges whenever the selection includes a border-adjacent cell, then inset by `gap_px`.

### Single-Monitor Configuration UI

- Provide a simple configuration surface with **minimal user input**:
  - Grid size selection: limited to sane presets based on monitor resolution/orientation (e.g., 2×2, 3×2, 4×3 …).
  - Gap size selection: either a small numeric field or a small set of presets (none / small / medium / large).
- Validation rules:
  - UI must **not allow** grid sizes that would violate the minimum cell size constraints given the current monitor resolution and orientation.
  - Provide clear, concise feedback when a chosen grid is invalid (“Cells would be smaller than 480×360 px; choose fewer rows/columns”).
- Persistence:
  - Save and load **single-monitor** grid configurations (including gap size and snapping flag) using the configuration module introduced in earlier phases (or a minimal JSON/registry-based layer if not yet present).

---

## [ADDED] Suggested Architecture & Modules

These changes should extend existing modules without breaking the overall layered design.

### Platform Layer

- `platform::monitors`:
  - Add helpers to expose **orientation** (landscape/portrait) for the active monitor.
  - Optionally surface rotation change events or provide a polling-friendly API.

### Domain Layer

- `domain::grid`:
  - Accept an orientation hint and grid size to compute cell geometry.
  - Expose an API to query whether a cell or selection touches a monitor edge.

- `domain::core` (Rect utilities):
  - Add helpers for:
    - Edge-snapping (expand rect to edges).
    - Gap insetting (shrink rect uniformly by `gap_px`, clamped to a minimum size).

### Config Layer (Single-Monitor Focus)

- Introduce or extend a simple schema for:
  - `single_monitor.grid_cols`, `single_monitor.grid_rows`.
  - `single_monitor.snapping_enabled`.
  - `single_monitor.gap_px`.
  - The last used/valid configuration for the active monitor.

### UI / Companion App

- Minimal configuration window:
  - Can be launched from the main app or run as a separate executable.
  - Reads/writes the same configuration used by the core app.
  - No complex live preview required for Phase 4; simple “Apply/Save” is enough.

---

## [ADDED] Phase 4 Milestones

### Milestone 1: Single-Monitor Hardening

- Audit all monitor-dependent logic to ensure it behaves correctly when only one monitor is present.
- Add explicit guards preventing accidental use of multi-monitor paths.
- Add tests/integration checks for:
  - Single-monitor enumeration.
  - Basic selection and positioning on various resolutions.

### Milestone 2: Rotation & Grid Adaptation

- Implement orientation detection and grid recomputation.
- Define a small, opinionated set of grid presets for landscape vs. portrait.
- Enforce updated minimum cell-size constraints for both orientations.

### Milestone 3: Snapping & Gaps

- Implement snapping helpers in the domain layer.
- Wire snapping into the final rectangle calculation before `platform::window::apply_rect`.
- Expose `snapping_enabled` and `gap_px` in configuration.

### Milestone 4: Single-Monitor Configuration UI

- Implement basic UI (or companion app) for:
  - Selecting grid size.
  - Enabling/disabling snapping.
  - Adjusting gap size.
  - Saving and loading single-monitor configurations.
- Add lightweight tests/integration checks to ensure settings persist and are honored by the main app.

---

## [ADDED] Phase 4 Exit Criteria

At the end of Phase 4, you should confidently be able to say:

- ✅ A single-monitor user can rely on tactile-win for daily work.
- ✅ Grid behavior adapts correctly when the monitor rotates between landscape and portrait.
- ✅ Optional snapping and gaps behave as expected and can be disabled.
- ✅ Invalid grid sizes are prevented by the configuration UI and validation logic.
- ✅ Single-monitor settings (grid, gap, snapping) are saved and restored correctly.
- ✅ No multi-monitor behavior is exposed yet; that is clearly deferred to Phase 5.

Multi-monitor support, per-monitor grid configurations, and broader polish (tray icon, help system, distribution) are now fully owned by **Phase 5** in `copilot-instructions_phase-5.md`.