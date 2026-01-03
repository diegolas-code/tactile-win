# Tactile-Win Phase 3: UI Overlay & Input Handling

**Implementation Guide for Interactive Grid Overlay System**

---

## Phase 3 Overview

Phase 3 implements the interactive user interface that brings together the Phase 1 infrastructure and Phase 2 domain logic into a working modal application. This phase creates the visual overlay system and input handling that allows users to select grid cells and position windows interactively.

### Core Objectives

1. **Global Hotkey System**: Register and handle system-wide keyboard shortcuts to enter modal mode
2. **Overlay Window Management**: Create transparent, topmost overlay windows with proper focus handling  
3. **Grid Visualization**: Render grid lines and keyboard letter labels on overlay windows
4. **Input Capture**: Capture keyboard input during modal mode without stealing focus
5. **Application State**: Manage transition between idle and selecting states
6. **Window Positioning**: Apply selection results to active window positioning

### Technical Foundation

Building on our completed infrastructure:
- **Phase 1**: DPI awareness, monitor enumeration, window management ✓
- **Phase 2**: Domain logic (keyboard → grid → selection) ✓  
- **Phase 3**: UI overlay + input handling (this phase)

---

## Architecture & Module Structure

### New Modules for Phase 3

```
src/
├── app/                    # Application orchestration (NEW)
│   ├── mod.rs             # Module exports
│   ├── state.rs           # Application state management
│   └── controller.rs      # Event handling and coordination
├── ui/                     # User interface (NEW)
│   ├── mod.rs             # Module exports
│   ├── overlay.rs         # Overlay window management
│   └── renderer.rs        # Grid and text rendering
├── input/                  # Input handling (NEW)
│   ├── mod.rs             # Module exports
│   ├── hotkeys.rs         # Global hotkey registration
│   └── keyboard.rs        # Modal keyboard capture
├── domain/                 # Existing - no changes
├── platform/               # Existing - no changes
└── main.rs                # Updated for application loop
```

### Module Responsibilities

#### `app/` - Application Orchestration
- **state.rs**: Application state machine (`Idle` ↔ `Selecting`)
- **controller.rs**: Coordinates between input, domain, UI, and platform layers

#### `ui/` - User Interface  
- **overlay.rs**: Creates overlay windows per monitor with proper Win32 styles
- **renderer.rs**: Renders grid lines and keyboard letters using graphics backend

#### `input/` - Input Management
- **hotkeys.rs**: Global hotkey registration and callback handling
- **keyboard.rs**: Modal keyboard capture during selection mode

---

## Implementation Plan

### Step 1: Application State Management

Create the core application state machine and controller:

**app/state.rs**
```rust
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AppState {
    Idle,
    Selecting(SelectingState),
}

#[derive(Debug, Clone, PartialEq, Eq)]  
pub struct SelectingState {
    pub active_monitor_index: usize,
    pub selection: Selection,
    pub selection_timeout: Option<Instant>, // 30s timeout
}
```

**Key Requirements:**
- State transitions triggered by hotkey press and selection completion
- Track which monitor is currently active during selection
- **Grid instances maintained in AppController** (stable configuration, not transient state)
- Thread-safe state sharing between input and UI components
- **Selection timeout**: 30-second automatic cancellation if no input

### Step 2: Global Hotkey System

Implement system-wide hotkey registration:

**input/hotkeys.rs**
```rust
pub struct HotkeyManager {
    hotkey_id: i32,
    hwnd: HWND, // Hidden message window
}

impl HotkeyManager {
    pub fn new() -> Result<Self, HotkeyError>;
    pub fn register_hotkey(&mut self, modifiers: u32, vk: u32) -> Result<(), HotkeyError>;
    pub fn unregister_hotkey(&mut self) -> Result<(), HotkeyError>;
    // Message pump integration for hotkey detection
}
```

**Technical Details:**
- Use `RegisterHotKey` Win32 API for global hotkey registration
- Create hidden message window to receive `WM_HOTKEY` messages
- Default hotkey: `VK_OEM_3` (backtick/tilde key) - easily accessible, rarely conflicts
- Handle registration failures gracefully (hotkey already in use)
- Clean unregistration on application exit

### Step 3: Overlay Window System

Create topmost, transparent overlay windows:

**ui/overlay.rs**
```rust
pub struct OverlayManager {
    overlays: Vec<OverlayWindow>,
}

pub struct OverlayWindow {
    hwnd: HWND,
    monitor_index: usize,
    grid: Grid,
    is_active: bool, // Shows letters vs just grid
}
```

**Critical Window Styles:**
```rust
// Extended styles for proper overlay behavior
WS_EX_TOPMOST | WS_EX_LAYERED | WS_EX_TRANSPARENT | 
WS_EX_NOACTIVATE | WS_EX_TOOLWINDOW

// Base styles  
WS_POPUP | WS_VISIBLE
```

**Key Requirements:**
- One overlay window per monitor
- `WS_EX_NOACTIVATE`: Prevents stealing focus from active window
- `WS_EX_TOPMOST`: Ensures visibility above other windows
- `WS_EX_LAYERED`: Enables transparency and alpha blending
- `WS_EX_TRANSPARENT`: Allows mouse events to pass through
- `WS_EX_TOOLWINDOW`: Prevents appearance in taskbar
- Per-monitor DPI awareness for proper positioning and scaling

**⚠️ CRITICAL: Overlay Input Responsibility**
- **Overlay windows NEVER process keyboard messages**
- **All input comes through KeyboardCapture only**
- **Overlay is 100% "dumb view" - pure rendering target**
- This prevents conflicts between `WS_EX_TRANSPARENT` + `WS_EX_NOACTIVATE` and input handling

### Step 4: Grid Rendering System

Implement graphics rendering for grid visualization:

**ui/renderer.rs**
```rust
// Separate layout calculation from rendering
pub struct GridLayout {
    lines: Vec<Line>,
    letters: Vec<LetterPosition>,
}

pub struct GridRenderer {
    // Graphics backend (tiny-skia recommended)
}

impl GridRenderer {
    pub fn calculate_layout(&self, grid: &Grid, active: bool) -> GridLayout;
    pub fn render_layout(&self, layout: &GridLayout) -> RenderedFrame;
}
```

**Separation of Concerns:**
- **GridLayout**: Pure calculation - lines, letter positions, bounding boxes (easily testable)
- **GridRenderer**: Pure rendering - takes layout and draws it (graphics-specific)

**Rendering Requirements:**
- **Grid Lines**: Semi-transparent white lines (2-3px width) 
- **Letter Labels**: High contrast, large font (24-32pt), centered in cells
- **Active Monitor**: Shows letters, inactive monitors show grid only
- **Transparency**: Background fully transparent, elements semi-transparent
- **Performance**: Render to bitmap, update only on state changes

**Graphics Backend Choice:**
- **Primary**: `tiny-skia` - Pure Rust, fast, good transparency support
- **Alternative**: Direct Win32 GDI calls if simpler integration needed
- **Avoid**: Direct2D/DirectWrite (complex setup, overkill for this use case)

### Step 5: Modal Keyboard Capture

Capture keyboard input during selection mode:

**input/keyboard.rs**
```rust
pub struct KeyboardCapture {
    hook: Option<HHOOK>, // Low-level keyboard hook
}

impl KeyboardCapture {
    pub fn start_capture(&mut self) -> Result<(), InputError>;
    pub fn stop_capture(&mut self) -> Result<(), InputError>;
    // Event handler for captured keys
}
```

**Input Handling Strategy:**
- **Low-level keyboard hook**: `SetWindowsHookEx` with `WH_KEYBOARD_LL`
- **Key filtering**: Only process valid grid keys (Q,W,E,A,S,D,Z,X,C,V) + navigation (arrows) + escape
- **Pass-through**: Allow all other keys to reach their intended targets
- **Focus preservation**: Active window remains focused throughout selection

**⚠️ CRITICAL: Threading and Hook Safety**
- **Hook callback runs on SYSTEM thread, NOT main thread**
- **NEVER mutate application state directly from hook callback**
- **Hook only posts events to main thread message queue**
- **All selection logic happens on main thread only**
- **This prevents deadlocks and race conditions**

```rust
// Hook callback example:
fn keyboard_proc(code: i32, wparam: WPARAM, lparam: LPARAM) -> LRESULT {
    if code >= 0 {
        // Parse key, validate, create event
        let event = KeyEvent::new(key);
        // POST to main thread - never call directly
        PostMessage(main_window, WM_USER + 1, event_data, 0);
    }
    CallNextHookEx(hook, code, wparam, lparam)
}
```

**Navigation Keys:**
- **Arrow keys**: Switch between monitors (Left/Right for horizontal navigation)
- **Escape**: Cancel selection, return to idle
- **Enter/Space**: Complete single-cell selection (same key twice)

### Step 6: Application Controller

Coordinate all components:

**app/controller.rs**  
```rust
pub struct AppController {
    state: Arc<Mutex<AppState>>,
    _hotkey_manager: HotkeyManagerGuard, // RAII wrapper
    _overlay_manager: OverlayManagerGuard, // RAII wrapper
    _keyboard_capture: KeyboardCaptureGuard, // RAII wrapper
    monitors: Vec<Monitor>,
    grids: Vec<Grid>, // Stable configuration, one per monitor
}

impl AppController {
    pub fn new() -> Result<Self, AppError>;
    pub fn run(&mut self) -> Result<(), AppError>; // Main event loop with timeout handling
    
    // Event handlers
    fn handle_hotkey(&mut self);
    fn handle_key_press(&mut self, key: char);
    fn handle_navigation(&mut self, direction: NavDirection);
    fn handle_selection_timeout(&mut self);
    fn apply_selection(&mut self);
}

// RAII Guards for guaranteed cleanup
pub struct HotkeyManagerGuard(HotkeyManager);
pub struct OverlayManagerGuard(OverlayManager);
pub struct KeyboardCaptureGuard(KeyboardCapture);

impl Drop for HotkeyManagerGuard {
    fn drop(&mut self) {
        let _ = self.0.unregister_hotkey(); // Guaranteed cleanup
    }
}
```

**Event Flow:**
1. **Hotkey pressed** → Enter selecting state → Show overlays → Start keyboard capture
2. **Valid key pressed** → Update selection → Update overlay rendering
3. **Selection complete** → Position window → Hide overlays → Return to idle
4. **Escape pressed** → Cancel selection → Hide overlays → Return to idle

---

## Technical Implementation Details

### Graphics Rendering Pipeline

**Overlay Window Rendering:**
1. Create overlay windows with proper styles per monitor
2. Set up graphics context (tiny-skia surface or GDI device context)  
3. Render grid lines based on Grid geometry
4. Render letter labels at cell centers using keyboard layout
5. Apply alpha blending for semi-transparency
6. Update only on state changes (selection progress, monitor switching)

**Rendering Strategy:**
- **Phase 3**: Full re-render on state changes (simple, reliable)
- **Future optimization**: Dirty rectangles, pre-rendered elements (only if performance issues)
- **Priority**: Correctness over premature optimization

### DPI and Multi-Monitor Handling

**DPI Awareness:**
- Use existing Phase 1 monitor enumeration with DPI information
- Scale grid line thickness and font sizes per monitor DPI
- Account for DPI differences when positioning overlays

**Multi-Monitor Support:**
- Create one overlay per monitor
- Active monitor shows letters, inactive show grid-only
- Arrow key navigation switches active monitor
- Selection restricted to single monitor initially (cross-monitor in Phase 4)

### Memory Management

**Resource Cleanup:**
- Proper cleanup of Win32 handles (overlay windows, keyboard hooks)
- Graceful handling of resource creation failures
- RAII patterns for automatic cleanup

**Error Recovery:**
- Continue operation if single overlay creation fails
- Fall back gracefully if graphics rendering fails
- Maintain basic functionality even with reduced features

### Thread Safety

**Concurrency Model:**
- Main thread: Win32 message pump and UI rendering
- Input callbacks: Run on system thread, post messages to main thread
- Shared state: Use `Arc<Mutex<AppState>>` for thread-safe access
- Avoid blocking operations on UI thread

---

## Testing Strategy

### Unit Testing

**Component Testing:**
- `app/state.rs`: State transition logic, all valid state changes
- `ui/renderer.rs`: Grid rendering output, letter positioning calculations  
- `input/keyboard.rs`: Key filtering logic, navigation handling
- Mock Win32 APIs where possible using dependency injection

### Integration Testing

**System Integration:**
- Hotkey registration and detection
- Overlay window creation and positioning
- Keyboard capture during modal mode
- Multi-monitor overlay coordination

**User Workflow Testing:**
- End-to-end selection scenarios: hotkey → key selection → window positioning
- Multi-monitor navigation: arrow keys between monitors
- Error scenarios: escape key, invalid keys, window positioning failures

### Manual Testing Protocol

**Test Cases:**
1. **Basic Selection**: Hotkey → single key → window positioned
2. **Multi-cell Selection**: Hotkey → first key → second key → window positioned  
3. **Navigation**: Hotkey → arrow keys → selection on different monitor
4. **Cancellation**: Hotkey → escape → return to idle without changes
5. **Error Handling**: No active window, maximized window, non-resizable window

**Validation Points:**
- Overlay appears on all monitors
- Active window maintains focus during selection
- Grid lines and letters render correctly at different DPI scales
- Window positioning works across different monitor configurations

---

## Error Handling & Edge Cases

### Input Edge Cases

**Invalid Key Sequences:**
- Invalid keys during selection → **silent ignore**, maintain current state
- Rapid key presses → debounce input, ignore duplicates within threshold
- System key combinations → pass through to system, don't interfere

**Error Categories:**
- **Silent errors**: Invalid keys, rapid input → ignore, continue selection
- **Visible errors**: No active window, non-resizable window → show brief message, return to idle
- **Timeout**: 30 seconds no input → automatic cancellation to idle

**Monitor Changes:**
- Monitor disconnected during selection → **cancel selection**, log event, return to idle
- DPI change during selection → **cancel selection**, log event, return to idle  
- Resolution change → **cancel selection**, log event, return to idle

**⚠️ Phase 3 Strategy:** For complex edge cases, prefer **graceful cancellation** over perfect handling. Log for debugging, but keep the code simple and robust.

### Window Management Edge Cases  

**Target Window Issues:**
- No active window → show error message, return to idle
- Non-resizable window → show error message, return to idle  
- Minimized window → restore window first, then position
- Full-screen application → may not work, graceful failure

**System Interaction:**
- Screen saver activation → cancel selection
- User switching → cancel selection, clean up hooks
- System shutdown → proper resource cleanup

### Resource Management

**Memory and Handle Limits:**
- Overlay window creation failure → continue with available overlays
- Graphics context creation failure → fall back to simpler rendering
- Keyboard hook registration failure → disable modal mode, show error

---

## Implementation Priority & Milestones

### Milestone 1: Basic State Management
- [ ] `app/state.rs`: Application state enum and transitions (simplified SelectingState)
- [ ] `app/controller.rs`: Basic controller with state management and grid storage
- [ ] RAII wrapper structs for resource cleanup
- [ ] Unit tests for state transitions
- **Validation**: State transitions work correctly in isolation

### Milestone 2: Global Hotkey
- [ ] `input/hotkeys.rs`: Global hotkey registration with RAII cleanup
- [ ] Hidden message window for hotkey reception
- [ ] Integration with controller for state transitions
- [ ] Test hotkey detection and state changes
- **Validation**: Hotkey toggles application state, proper cleanup on exit

### Milestone 3: Basic Overlay
- [ ] `ui/overlay.rs`: Overlay window creation with proper styles (passive view only)
- [ ] Per-monitor overlay positioning with DPI awareness
- [ ] Show/hide overlay based on application state
- [ ] Verify no keyboard message processing in overlays
- **Validation**: Transparent overlays appear on all monitors, input ignored

### Milestone 4: Grid Rendering
- [ ] `ui/renderer.rs`: Separate GridLayout calculation from rendering
- [ ] Grid line rendering (simple, full re-render approach)
- [ ] Letter label rendering with proper positioning
- [ ] Active vs inactive monitor rendering states
- **Validation**: Visual grid with letters appears correctly

### Milestone 5: Keyboard Capture
- [ ] `input/keyboard.rs`: Modal keyboard input capture with proper threading
- [ ] Hook callback that only posts events to main thread
- [ ] Key filtering and selection processing on main thread
- [ ] Navigation key handling (arrows, escape) with timeout
- [ ] Selection timeout (30s) implementation
- **Validation**: Keys captured during modal mode, no threading issues

### Milestone 6: Window Positioning Integration  
- [ ] Connect selection completion to window positioning
- [ ] Differentiated error handling (silent vs visible)
- [ ] End-to-end workflow testing
- [ ] Edge case handling with graceful cancellation
- **Validation**: Complete user workflow from hotkey to positioned window

---

## Phase 3 Success Criteria

### Functional Requirements
✅ **Global hotkey activates modal mode**
✅ **Overlay appears on all connected monitors**  
✅ **Grid lines and letters render correctly**
✅ **Keyboard input captured during selection (hook threading safe)**
✅ **Valid key presses progress selection state**
✅ **Arrow keys navigate between monitors**
✅ **Selection completion positions active window**
✅ **Escape key cancels selection**
✅ **30-second timeout cancels selection automatically**

### Technical Requirements  
✅ **DPI awareness maintained across all monitors**
✅ **Active window retains focus during selection**
✅ **Overlay windows don't process keyboard input (passive view)**
✅ **Resource cleanup guaranteed via RAII wrappers**
✅ **Hook callbacks never mutate state directly (thread safety)**
✅ **Error handling differentiated (silent vs visible)**
✅ **Complex edge cases handled via graceful cancellation**

### Performance Requirements
✅ **Modal activation within 100ms of hotkey press**
✅ **Smooth rendering without visible lag (simple full re-render)**
✅ **Minimal impact on system performance when idle**
✅ **No deadlocks or race conditions in hook handling**

---

## Phase 4 Preparation

Phase 3 establishes the foundation for advanced features in Phase 4:

- **Cross-monitor Selection**: Extend selection to span multiple monitors
- **Configuration UI**: Settings for grid sizes, hotkeys, appearance  
- **Animation**: Smooth transitions and visual feedback
- **Advanced Features**: Window snapping, saved layouts, system tray integration

The modular architecture from Phase 3 will support these extensions without major refactoring.

---

**End of Phase 3 Implementation Guide**

This phase transforms the application from a working proof-of-concept into a functional, interactive window positioning tool. The combination of Phase 1's infrastructure, Phase 2's domain logic, and Phase 3's interactive UI creates a complete, polished application ready for daily use.