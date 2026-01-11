# Phase 5: Multi-Monitor & Polish

> NOTE FOR AI AGENTS AND EDITORS: Any ~~struck-through~~ text in this project’s documentation represents outdated guidance. Follow sections explicitly marked with [ADDED] and the new Phase 4/5 split described here.

~~Previous split: Phase 4A (advanced core features) and 4B (polish)~~

**Status**: Phase 4 (single-monitor MVP) Complete ✓  
**Prerequisite**: Stable single-monitor behavior with rotation-aware grids, optional snapping, and basic configuration UI  
**Goal**: [ADDED] Extend tactile-win to **multi-monitor setups** and deliver the remaining polish features (tray, help, visual refinement, distribution).

---

## [ADDED] Phase 5 Overview

Phase 5 builds on the robust single-monitor MVP from Phase 4 and focuses on:

1. Full **multi-monitor support** (selection, movement, and per-monitor configuration).
2. **Advanced configuration UI** that is multi-monitor aware.
3. **Polishing features** such as system tray integration, help, and visual refinements.

### [ADDED] Scope Boundary

- In scope:
  - Multi-monitor operation on the main Windows desktop.
  - Cross-monitor window movement with stable monitor identification.
  - Per-monitor grid configuration and persistence.
  - Tray, help, and basic distribution mechanics.
- Out of scope (for now):
  - Complex virtual desktop integrations.
  - Non-standard display topologies beyond what Windows exposes via standard APIs.

---

## [ADDED] Core Objectives

1. **Multi-Monitor Window Management**  
   Enable window positioning and selection across multiple physical monitors within the main desktop, handling different resolutions and DPIs.

2. **Per-Monitor Grids & Configuration**  
   Allow users to configure grid size, snapping, and gaps **individually per monitor**, and persist those settings.

3. **Advanced Configuration UI (Multi-Monitor Aware)**  
   Provide a configuration experience that clearly represents multiple monitors and allows easy switching and editing.

4. **Polish & System Integration**  
   Add practical features such as a system tray icon, basic help, and initial packaging/distribution support.

---

## [ADDED] Functional Requirements

### Multi-Monitor Support

- The application must:
  - Enumerate all connected monitors and assign a stable `MonitorId` to each.
  - Support cross-monitor selection and movement between **horizontally adjacent** monitors at minimum.
  - Correctly account for per-monitor DPI scaling and the shared virtual desktop coordinate space.
  - Handle monitor hot-plug events gracefully (connect/disconnect, resolution/orientation changes).

### Per-Monitor Configuration

- For each monitor (identified by `MonitorId`), users must be able to:
  - Configure grid size (rows/columns) with validation per monitor.
  - Configure snapping enable/disable and gap size independently.
  - Save and load multi-monitor configurations (e.g., across reboots).

- Configuration persistence:
  - Use a schema that stores per-monitor entries keyed by stable `MonitorId`.
  - Provide migration/cleanup when monitors are removed or IDs change.

### Advanced Configuration UI

- UI requirements:
  - Show a **visual list or map of monitors**, clearly indicating which one is selected for editing.
  - Allow switching between monitors with a simple interaction (click, dropdown, tabs, etc.).
  - For each monitor, expose:
    - Grid size controls.
    - Snapping toggle and gap controls.
    - Any additional per-monitor flags needed.
  - Offer **save/load** capabilities for multi-monitor layouts.

### Polish Features

- System tray integration:
  - Tray icon with at least:
    - Enable/disable toggle.
    - Open configuration UI.
    - Exit application.

- Help / onboarding:
  - Simple help overlay or link from configuration explaining:
    - Hotkey.
    - Basic selection flow.
    - Multi-monitor behavior.

- Visual refinements:
  - Minor overlay visual tweaks (colors, spacing, font sizes) based on real-world use.

- Distribution basics:
  - A minimal but functional way to distribute the app (e.g., portable zip or basic installer script).

---

## [ADDED] Suggested Architecture & Modules

> Much of this is adapted from the previous Phase 4A/4B split, now consolidated as Phase 5.

### Platform Layer

- `platform::monitors` & `platform::multi_monitor` (new/extended):
  - Provide stable `MonitorId` values using Windows display configuration APIs.
  - Expose monitor positions, DPI, and orientation in the virtual desktop coordinate space.

- `platform::window` & `platform::window_constraints` (optional/extended):
  - Handle cross-monitor movement while respecting window constraints.

### Domain Layer

- `domain::cross_monitor` (new):
  - Encapsulate logic for cross-monitor selections (initially horizontal adjacency only).
  - Calculate combined rectangles when selections span multiple monitors.

- `domain::selection_history` (optional but useful):
  - Track recent selections across monitors for future UX enhancements.

### Config Layer

- Extend the configuration schema introduced in earlier phases to:
  - Store per-monitor grid/snapping/gap settings keyed by `MonitorId`.
  - Optionally store defaults for new/unseen monitors.

### Input / Navigation

- `input::navigation` (new/extended):
  - Support monitor navigation via keyboard (e.g., Left/Right arrows, Tab) during selection.
  - Ensure navigation respects the real left-to-right ordering based on monitor layout.

### UI Layer

- Multi-monitor configuration UI:
  - Build on the single-monitor configuration UI from Phase 4.
  - Add a monitor selector with per-monitor panels.

- System tray & help:
  - `ui::system_tray` for tray icon integration.
  - `ui::help_overlay` or a simple help dialog.

---

## [ADDED] Phase 5 Milestones

### Milestone 1: Multi-Monitor Foundation

- Implement stable `MonitorId` generation and multi-monitor enumeration.
- Establish left-to-right ordering and basic layout structure.
- Wire selection and window movement to use `MonitorId` instead of raw indices.

### Milestone 2: Cross-Monitor Selection & Movement

- Implement cross-monitor selection for horizontally adjacent monitors.
- Ensure rectangle calculations handle differing resolutions and DPIs.
- Add guardrails and clear feedback for unsupported layouts (e.g., complex topologies).

### Milestone 3: Per-Monitor Configuration

- Extend the configuration schema to per-monitor entries.
- Implement save/load for multi-monitor grid configurations.
- Update the configuration UI to present and edit settings per monitor.

### Milestone 4: Tray, Help, and Visual Polish

- Add a system tray icon with core actions (enable/disable, open config, exit).
- Provide a simple help experience for new users.
- Apply minor overlay/UI refinements based on feedback.
- Add basic distribution artifacts (e.g., scripts or notes for building a portable package or installer).

---

## [ADDED] Phase 5 Exit Criteria

At the end of Phase 5, you should confidently be able to say:

- ✅ Multi-monitor setups are fully supported with correct behavior across different resolutions and DPIs.
- ✅ Users can configure grid/snapping/gap settings **individually per monitor**.
- ✅ Multi-monitor configurations can be saved and restored reliably.
- ✅ The app provides a basic but usable tray and help experience.
- ✅ The app can be distributed in at least one practical form (portable or installer).

Phase 5 completes the transition from a solid single-monitor MVP to a polished, multi-monitor-aware window management tool suitable for daily use on more complex setups.
