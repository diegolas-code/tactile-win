# Phase 2 Recommendations Synthesis: Domain Logic

## 🎯 Central Objective
> *"Implement pure, testable domain logic for grid-based window positioning using QWERTY keyboard layout, supporting single and multi-cell selections with proper coordinate calculations"*

Phase 2 builds the **pure business logic** on top of Phase 1's solid infrastructure.

---

## 1. CRITICAL DECISION: Grid Coordinate System
**Before implementing grids**, define the coordinate model:

- **Grid coordinates**: (row, col) starting from (0,0) at top-left
- **Monitor coordinates**: Real pixels from Platform Layer
- **Cell coordinates**: Grid position to pixel rectangle conversion
- **Selection coordinates**: Start/end cell pairs forming selection rectangles

**Fundamental principle**: Grid logic is **completely pure** - no Win32, no DPI, just math.

---

## 2. QWERTY Layout Mapping (CRITICAL - Foundation)
**This must be precisely defined** before cell selection:

**Standard 3x2 Layout**:
```
Q W E
A S D
```

**Extended 4x2 Layout**:
```
Q W E R
A S D F
```

**Key Requirements**:
- **Row-major mapping**: Q=0, W=1, E=2, A=3, S=4, D=5 (internal: `index = row * cols + col`)
- **API Design**: Public interface returns `(row, col)` coordinates, never flat indices
- **Case insensitive**: Both 'q' and 'Q' map to same cell
- **Invalid key handling**: Keys not in layout are rejected
- **Extensible design**: Support different grid sizes without code changes

---

## 3. Grid Structure (Mental Model)
Before code, define what a "Grid" represents:
```rust
// Conceptual - no code yet
Grid {
    cols: u32,           // Number of columns (e.g., 3)
    rows: u32,           // Number of rows (e.g., 2)  
    cell_width: i32,     // Width of each cell in pixels
    cell_height: i32,    // Height of each cell in pixels
    offset_x: i32,       // Grid position within monitor (from work_area)
    offset_y: i32,       // Grid position within monitor (from work_area)
    keyboard_layout: KeyboardLayout,  // QWERTY mapping
}
```

**Critical**: The grid origin is always the top-left of the monitor work area provided by the platform layer. The grid does not decide where it's located, only represents the layout within that area.

---

## 4. Specific Domain Modules

### 4.1 domain::keyboard
**Single responsibility**: "Convert keyboard input to grid coordinates"

**Concrete tasks**:
- QWERTY layout definition and validation
- Key character → (row, col) conversion
- Support for multiple grid sizes (3x2, 4x2, etc.)
- Invalid key detection and error handling
- **Critical**: Must be completely testable without Win32

### 4.2 domain::grid  
**Single responsibility**: "Manage grid geometry and cell calculations"

**Concrete tasks**:
- Grid creation from monitor work area
- Cell index → pixel rectangle conversion
- Grid size validation (minimum cell sizes)
- Grid positioning within monitor bounds
- **Critical**: All calculations in real pixels (DPI-aware from Platform)

### 4.3 domain::selection
**Single responsibility**: "Handle two-step selection process and bounding boxes"

**Concrete tasks**:
- Selection state management (None → Start → Complete)
- Multi-cell bounding box calculation  
- Selection normalization (Q→S = S→Q)
- Single-cell selection handling (Q→Q)
- Selection validation and error cases

---

## 5. Selection Logic Requirements

### Two-Step Selection Process
**Critical behavior** that must be implemented correctly:

1. **Initial State**: No selection active
2. **First Key Press**: Records start cell, highlights it
3. **Second Key Press**: Records end cell, calculates bounding rectangle
4. **Completion**: Returns final rectangle for window positioning

### Bounding Box Calculation
**Example for 3x2 grid**:
```
Q W E  →  (0,0) (0,1) (0,2)
A S D  →  (1,0) (1,1) (1,2)

Selection Q→S: 
- Start: (0,0), End: (1,1)  
- Bounding box: top_left=(0,0), bottom_right=(1,1)
- Covers cells: Q, W, A, S
```

### Selection Validation
Must handle these cases correctly:
- ✅ **Valid selections**: Q→S, W→A, E→Q (any two valid keys)
- ✅ **Single cell**: Q→Q, S→S (same key twice)  
- ✅ **Order independence**: Q→S = S→Q (same result)
- ❌ **Invalid keys**: Numbers, symbols, unmapped letters
- ❌ **Mixed grids**: Start on one grid, end on different grid

**Note**: Mixed grid detection is handled by the orchestration layer (Phase 3), not domain logic. Domain modules assume single-grid operations.

---

## 6. Implementation Architecture

### 6.1 Module Dependencies
```
domain::keyboard  ←  domain::grid  ←  domain::selection
      ↑                    ↑               ↑
    Pure                 Pure            Pure
   (no I/O)            (no I/O)        (no I/O)
```

### 6.2 Data Flow
```
Keyboard Input → domain::keyboard → Grid Coordinates
                      ↓
Monitor Work Area → domain::grid → Cell Rectangles  
                      ↓
Grid Coordinates → domain::selection → Window Rectangle
```

### 6.3 Error Handling Strategy
**Fail fast with specific errors**:
- `KeyboardError::InvalidKey(char)` 
- `GridError::CellTooSmall { actual: (i32, i32), required: (i32, i32) }`
- `SelectionError::InvalidCell { row: u32, col: u32 }`
- `SelectionError::IncompleteSelection`

---

## 7. Implementation Order

### Step 1: Keyboard Layout (Foundation)
1. **Define QWERTY layout constants** for 3x2 default grid
2. **Implement key→coordinate mapping** with comprehensive tests  
3. **Handle invalid keys** with proper error types
4. **Validate extensibility** for different grid sizes

### Step 2: Grid Geometry
5. **Grid creation** from monitor work area dimensions
6. **Cell calculation** - convert grid coordinates to pixel rectangles
7. **Grid validation** - ensure minimum cell sizes are met
8. **Positioning logic** - handle grid placement within monitor

### Step 3: Selection Logic
9. **Selection state machine** - None → Start → Complete → None
10. **Bounding box calculation** with proper normalization
11. **Edge case handling** - single cells, order independence
12. **Integration testing** - keyboard + grid + selection

---

## 8. Testing Strategy

### Unit Tests (Each module isolated)
- **domain::keyboard**: Every possible key mapping, invalid inputs
- **domain::grid**: Grid calculations with various monitor sizes  
- **domain::selection**: All selection combinations and edge cases

### Integration Tests (Modules combined)
- **Full selection workflows**: Q→S on different grid sizes
- **Error propagation**: Invalid selections through the pipeline
- **Real monitor scenarios**: Using actual monitor dimensions

### Property-Based Testing (Advanced)
- **Grid calculations**: Cell coordinates must always be within bounds
- **Selection invariants**: Q→S always equals S→Q
- **Bounding boxes**: Must always contain all selected cells

**Technical Debt**: Property-based testing should be implemented later as the domain logic stabilizes. Mark as explicit future enhancement.

---

## 9. Critical Validation Points

At the end of Phase 2, you should be able to:
- ✅ **Convert any valid key to grid coordinates** (Q → (0,0))
- ✅ **Calculate pixel rectangles for any cell** ((0,0) → Rect{x,y,w,h})  
- ✅ **Handle complete selections** (Q→S → window rectangle)
- ✅ **Reject invalid input gracefully** (numbers, unmapped keys)
- ✅ **Support multiple grid sizes** (3x2, 4x2, etc.)

### Example Integration Test
```rust
let monitor = /* Monitor with 1920x1080 work area */;
let grid = Grid::new(3, 2, &monitor.work_area)?;
let layout = QwertyLayout::new(3, 2);

// Test complete selection workflow
let start_coords = layout.key_to_coords('Q')?;  // (0,0)
let end_coords = layout.key_to_coords('S')?;    // (1,1)
let selection = Selection::from_coords(start_coords, end_coords)?;
let window_rect = grid.selection_to_rect(&selection)?;

assert_eq!(window_rect, Rect::new(0, 0, 1280, 720)); // Left 2/3 of monitor
```

---

## 10. Phase 2 Exit Checklist

When finished, you should confidently say:
- ✅ I can map any QWERTY key to grid coordinates
- ✅ I can calculate exact pixel rectangles from grid coordinates  
- ✅ I can handle two-step selections with proper bounding boxes
- ✅ I can validate grid sizes against monitor constraints
- ✅ All domain logic is pure and extensively tested
- ✅ Error handling is comprehensive and specific

---

## 11. What NOT to Implement Yet

🚫 **Phase 3 concerns (avoid these temptations)**:
- Global hotkey registration
- Overlay window creation or rendering  
- Keyboard input capture
- Window positioning integration
- Multi-monitor logic
- Any Win32 API calls in domain modules

If you're thinking about hotkeys or overlays, you're getting ahead of yourself.

---

## 13. Recommended Starting Point

**👉 Start with `domain::keyboard` first** - do not implement all modules at once.

### Why Start with Keyboard Layout?
- **Completely independent** - no dependencies on other Phase 2 modules
- **Defines system vocabulary** - establishes the foundation for all grid operations
- **Enables rapid testing** - pure functions with clear inputs/outputs
- **Forces good design** - must handle extensibility and error cases upfront

### Implementation Sequence:
1. **First**: `domain::keyboard` - key mapping and layout logic
2. **Second**: `domain::grid` - builds naturally on keyboard coordinates  
3. **Third**: `domain::selection` - integrates keyboard + grid seamlessly

**Anti-pattern**: Starting with grid geometry leads to uncertainty about keyboard integration.

---

## Integration with Phase 1

Phase 2 **uses** Phase 1 infrastructure but **does not modify** it:

### Uses from Phase 1:
- `platform::monitors::Monitor.work_area` → Grid dimensions
- `domain::core::Rect` → Cell rectangles and selections
- `platform::window::position_window()` → Final window positioning

### Provides to Phase 3:
- Complete keyboard→rectangle conversion pipeline
- Validated grid geometry calculations  
- Pure selection logic ready for UI integration

---

## Expected Result

A **complete, pure domain layer** where:
1. You can type 'Q' and get precise grid coordinates
2. You can type 'Q' then 'S' and get an exact window rectangle
3. All logic is thoroughly tested and Win32-independent
4. Error handling covers all edge cases
5. Multiple grid sizes are supported

With this foundation, Phase 3 (UI and interaction) becomes straightforward orchestration of well-tested components.

---

## Sources
- copilot-instructions.md functional specifications
- Phase 1 infrastructure implementation
- QWERTY keyboard layout standards
- Grid-based window management best practices