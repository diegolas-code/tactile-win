# Phase 4: Single-Monitor MVP & Configuration

> NOTE FOR AI AGENTS AND EDITORS: Any ~~struck-through~~ text in this project’s documentation represents outdated guidance. Follow sections explicitly marked with [ADDED] and the new Phase 4/5 split described here.

~~Previous title: Phase 4A: Advanced Core Features (multi-monitor focused)~~

**Status**: Phases 1–3 Complete ✓  
**Prerequisite**: Working single-monitor prototype with keyboard capture, overlay, and basic positioning  
**Goal**: [ADDED] Deliver a production-ready MVP for **single-monitor setups only**, including rotation-aware grids and basic configuration UI.

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

3. **Basic Single-Monitor Configuration UI**  
   Provide a minimal but robust way for users to configure the single-monitor setup:
   - Grid size selection based on monitor resolution/orientation.
   - Validation to prevent invalid configurations.
   - Persistence of user preferences.

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

### Single-Monitor Configuration UI

- Provide a simple configuration surface with **minimal user input**:
  - Grid size selection: limited to sane presets based on monitor resolution/orientation (e.g., 2×2, 3×2, 4×3, etc.).
  - Minimum cell size configuration for validation purposes.
- Validation rules:
  - UI must **not allow** grid sizes that would violate the minimum cell size constraints given the current monitor resolution and orientation.
  - Provide clear, concise feedback when a chosen grid is invalid ("Cells would be smaller than 300×300 px; choose fewer rows/columns").
- Persistence:
  - Save and load **single-monitor** grid configurations using the configuration module introduced in earlier phases (or a minimal JSON/registry-based layer if not yet present).

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
  - Validate grid configurations based on minimum cell size requirements.

- `domain::core` (Rect utilities):
  - Existing rectangle utilities are sufficient for Phase 4.

### Config Layer (Single-Monitor Focus)

- Introduce or extend a simple schema for:
  - `single_monitor.grid_cols`, `single_monitor.grid_rows`.
  - `single_monitor.min_cell_width`, `single_monitor.min_cell_height`.
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

### Milestone 3: Single-Monitor Configuration UI

- Implement basic UI for:
  - Selecting grid size.
  - Adjusting minimum cell size constraints.
  - Saving and loading single-monitor configurations.
- Add lightweight tests/integration checks to ensure settings persist and are honored by the main app.

---

## [ADDED] Phase 4 Exit Criteria

At the end of Phase 4, you should confidently be able to say:

- ✅ A single-monitor user can rely on tactile-win for daily work.
- ✅ Grid behavior adapts correctly when the monitor rotates between landscape and portrait.
- ✅ Invalid grid sizes are prevented by the configuration UI and validation logic.
- ✅ Single-monitor settings (grid dimensions and cell constraints) are saved and restored correctly.
- ✅ No multi-monitor behavior is exposed yet; that is clearly deferred to Phase 5.

Multi-monitor support, per-monitor grid configurations, and broader polish (tray icon, help system, distribution) are now fully owned by **Phase 5** in `copilot-instructions_phase-5.md`.