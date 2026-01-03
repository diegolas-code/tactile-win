# Phase 1 Recommendations Synthesis: Infrastructure

## 🎯 Central Objective
> *"Be able to describe the Windows desktop correctly, stably, and DPI-aware, and move a window to a given rectangle without surprises"*

If this works, everything else is "just" orchestration.

---

## 1. CRITICAL DECISION: Coordinate System
**Before writing any code**, clearly define the units:

- **Platform layer**: Works with what Windows returns (RECT + mixed DPI)
- **Domain layer**: Works **exclusively** in real pixels, already normalized
- **Translation**: `platform::monitors` converts everything to absolute coordinates in real pixels

**Fundamental principle**: The domain must not know that DPI exists.

---

## 2. DPI Awareness (CRITICAL - First step)
**This must be the first thing** you implement:

**Option A (Recommended)**: `SetProcessDpiAwarenessContext` in `main.rs`
**Option B**: `.manifest` file in compilation

Without this, Windows will "stretch" coordinates and everything will be wrong.

---

## 3. Monitor Structure (Mental Model)
Before code, define what a "Monitor" is for your app:
```rust
// Conceptual - no code yet
Monitor {
    id: Stable Handle/index
    physical_rect: Total rectangle in real pixels
    work_area: Area without taskbar in real pixels  
    dpi_scale: Scale factor (1.0, 1.25, 1.5, etc.)
    raw_dpi: Original DPI X/Y
}
```

---

## 4. Specific Submodules

### 4.1 platform::monitors
**Single responsibility**: "Give me a reliable and DPI-aware `Vec<Monitor>`"

**Concrete tasks**:
- `EnumDisplayMonitors` for enumeration
- `GetDpiForMonitor` for per-monitor scale
- `GetMonitorInfo` for work area
- **Conversion**: Logical RECT → coherent real pixels
- **DO NOT** implement grids here

### 4.2 platform::window
**Single responsibility**: "If you give me an absolute rectangle, I place the window there"

**Concrete tasks**:
- `GetForegroundWindow` for active window (beware of child windows)
- Verify if it's resizable
- `SetWindowPos` with correct flags (without changing Z-order or focus)

### 4.3 domain::core (new)
**Pure domain** for basic rectangles:
```rust
// Conceptual
Rect { x, y, w, h }  // Always in real pixels
// Operations: normalization, intersection, bounding box
```

---

## 5. Early Validations (No UI)
In Phase 1 you should already be able to answer:
- ✅ Does this monitor support 3×2 grid with cells ≥ 480×360?
- ✅ Is this monitor <600px height and should be rejected?
- ✅ Does the work area affect the calculation?

---

## 6. Initial Project Configuration

### Cargo.toml dependencies:
```toml
[dependencies]
windows = { version = "0.52", features = [
    "Win32_UI_HiDpi",
    "Win32_Graphics_Gdi", 
    "Win32_UI_WindowsAndMessaging",
    "Win32_Foundation"
]}
```

### Implementation order:
1. **DPI Awareness** (main.rs)
2. **Monitor enumeration** (platform::monitors)
3. **Basic domain** (domain::core - rectangles)
4. **Window management** (platform::window)
5. **Size validation** (integration)

---

## 7. WHAT NOT TO THINK ABOUT YET
🚫 **Temptations you must avoid**:
- Hotkeys
- Overlay/UI
- QWERTY letters
- Multi-cell selection
- Multi-monitor selection

If you're thinking about letters, you're getting ahead of yourself.

---

## 8. Phase 1 Exit Checklist
When finished, you should be able to say:
- ✅ I can list all monitors with correct size and DPI
- ✅ I know the absolute coordinates of each one  
- ✅ I can move an arbitrary window to a given rectangle
- ✅ I reject invalid monitors/configurations
- ✅ Everything works with mixed DPI

---

## 9. Windows Virtual System Considerations
**Important to understand**:
- Primary monitor: usually at (0,0)
- Secondary monitors: can have **negative** coordinates
- Each monitor: independent DPI
- Work area ≠ total resolution (due to taskbar)

---

## Expected Result
A **solid and reliable** foundation where:
1. You know exactly what monitors there are
2. You know their real dimensions (DPI-aware)
3. You can move windows with precision
4. You have working size validations

With this working, the app is already **viable** - everything else will be orchestration.

---

## Sources
- copilot-instructions.md specifications document
- Consultations with multiple specialized AIs
- Win32 development best practices with Rust
- DPI-aware application development experience