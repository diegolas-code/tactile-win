# Phase 4A: Advanced Core Features

**Status**: Phase 3 Complete ✓  
**Prerequisite**: Working single-monitor MVP with keyboard capture and window positioning  
**Goal**: Advanced window management with multi-monitor support and user configuration
**Follow-up**: Phase 4B for professional polish and production readiness

---

## Phase 4A Overview

Phase 4A builds the advanced core functionality on top of the single-monitor MVP. This phase focuses on the technical challenges of multi-monitor support, enhanced selection logic, and persistent user configuration.

**Phase Split Rationale**: The original Phase 4 scope was substantial (10-16 days). Splitting into 4A (core features) and 4B (polish) creates a natural break point and allows shipping a fully functional advanced tool after 4A completion.

### Current Implementation Status

After Phase 3 completion:
- ✅ Single-monitor grid selection and positioning
- ✅ Global hotkey activation (Ctrl+Alt+T)
- ✅ Modal keyboard capture with low-level hooks
- ✅ Visual grid overlays with tiny-skia rendering
- ✅ DPI-aware coordinate handling
- ✅ RAII resource management throughout

### Phase 4A Objectives

1. **Multi-Monitor Window Management**: Enable window positioning across multiple physical monitors within the main desktop
2. **Enhanced Selection Logic**: Advanced selection modes with window constraints and history
3. **Persistent Configuration**: User-customizable grids, hotkeys, and preferences with registry/file storage

**Out of Scope for 4A** (deferred to Phase 4B):
- System tray integration
- Help system and tutorials
- Visual polish and animations
- Production distribution and installers

---

## Milestone Overview

### Milestone 1: Simple Multi-Monitor Support (Medium Complexity)
**Estimated Effort**: 2-3 days  
**Focus**: Basic cross-monitor window positioning with stable monitor identification

### Milestone 2: Advanced Selection Logic (Medium Complexity)
**Estimated Effort**: 2-3 days  
**Focus**: Enhanced selection modes and window constraints

### Milestone 3: Persistent Configuration System (Medium Complexity)
**Estimated Effort**: 2-3 days  
**Focus**: User settings storage and customization interface

**Total Phase 4A Effort**: 6-9 days

**Phase 4B Preview**: System tray, help system, visual polish, and production distribution (4-7 days)

---

## Milestone 1: Multi-Monitor Support

### Technical Challenges

**Multi-Monitor Coordinate Handling**
- Windows positions multiple monitors in a shared coordinate space
- Primary monitor typically at (0,0), secondary monitors positioned relative to it
- Need to handle left/right positioned secondary monitors correctly

**DPI Scaling Differences**
- Each monitor can have different DPI scaling (100%, 125%, 150%, 200%+)
- Grid cells must maintain consistent visual size across different DPI monitors
- Window positioning must account for per-monitor DPI when moving between displays

**Stable Monitor Identification**
- Monitor indices can change with hot-plug events or system restarts
- Need stable identifiers for persistent configuration
- Handle monitor disconnection/reconnection gracefully

### Implementation Tasks

#### Task 1.1: Multi-Monitor Coordinate System
**File**: `src/platform/multi_monitor.rs` (new)

**Purpose**: Replace monitor indices with stable `MonitorId` and handle multi-monitor layouts

**Key Components**:
- `MonitorId` - Stable identifier based on device path/hardware ID
- `MultiMonitorLayout` - Manages collection of monitors with their relationships
- `MonitorInfo` - Individual monitor data including DPI, position, and grid

**Core Functionality**:
- Generate stable monitor IDs that persist across system changes
- Track monitor positioning (left/right relationships)
- Convert between grid coordinates and screen coordinates
- Detect horizontally adjacent monitors

**Implementation Notes**:
- Use Windows `GetDisplayConfigBufferSizes`/`QueryDisplayConfig` APIs for stable IDs
- Fall back to resolution+position hash for legacy displays
- Maintain left-to-right ordering for navigation
- Handle DPI scaling per monitor

#### Task 1.2: Simple Monitor Management
**File**: `src/platform/monitors.rs` (extend existing)

**Purpose**: Detect and manage monitor configuration changes

**Key Features**:
- Enumerate monitors in left-to-right order
- Detect hot-plug events (monitor connect/disconnect)
- Simple primary/secondary classification
- Handle monitor arrangement changes gracefully

**Integration**: Extends existing monitor enumeration with stable ID support
```

#### Task 1.3: Simple Cross-Monitor Selection
**File**: `src/domain/cross_monitor.rs` (new)

**Purpose**: Enable window selection spanning horizontally adjacent monitors

**Policy**: Phase 4 supports only `AdjacentHorizontal` - left/right monitors only

**Key Features**:
- Track selection start/end across monitors
- Validate selections are between adjacent monitors only
- Calculate combined screen rectangle for cross-monitor spans
- Reject vertical or non-adjacent monitor selections

**UX Constraint**: Clear error feedback when invalid cross-monitor selections attempted


```

#### Task 1.4: Simple Monitor Navigation
**File**: `src/input/navigation.rs` (new)

**Purpose**: Enable navigation between monitors during selection

**Navigation Types**:
- Left/Right arrow keys to move between adjacent monitors
- Tab key to cycle through available monitors
- Return to primary monitor shortcut

**Constraints**: Only horizontal movement between adjacent monitors

**Integration**: Works with existing keyboard capture system
```

### Integration Points

#### State Machine Updates
**File**: `src/app/state.rs` (extend existing)

**Changes Required**:
- Replace `usize` monitor index with `MonitorId` in `SelectingState`
- Add `cross_monitor_policy` field for policy enforcement
- Add events for monitor layout changes and navigation
- Handle monitor hot-plug during active selection
```

#### Controller Integration
**File**: `src/app/controller.rs` (extend existing)

**New Methods Required**:
- Handle monitor navigation during selection mode
- Apply cross-monitor selections with policy validation
- Respond to monitor configuration changes
- Update overlays when active monitor changes
```

### Testing Strategy

**Focus Areas**:
- Monitor coordinate conversion accuracy
- Cross-monitor selection validation 
- Monitor hot-plug handling
- Configuration persistence across restarts

---

## Milestone 2: Advanced Selection Logic

### Enhanced Selection Features

#### Smart Window Constraints
- Respect minimum/maximum window sizes
- Handle non-resizable windows gracefully
- Maintain aspect ratios for specific window types

#### Multi-Selection Modes
- Hold Shift for additive selection (select multiple rectangles)
- Ctrl+Click for quick single-cell selection
- Tab to cycle through recent selections

### Implementation Tasks

#### Task 2.1: Window Constraint Detection
**File**: `src/platform/window_constraints.rs` (new)

**Purpose**: Detect and respect window size constraints when positioning

**Constraint Types**:
- Minimum/maximum window sizes
- Non-resizable windows (dialogs)
- Aspect ratio requirements

**Behavior**: Apply constraints to target rectangle before positioning window

#### Task 2.2: Selection History
**File**: `src/domain/selection_history.rs` (new)

**Purpose**: Track recent window selections for quick reuse

**Features**:
- Store recent selections with monitor, rect, window title, timestamp
- Limited history size (e.g., last 10 selections)
- Find similar selections for same window or size
- Use stable `MonitorId` for persistence across monitor changes

**Integration**: Basic foundation for future quick-select features

#### Task 2.3: Essential Keyboard Handling
**File**: `src/input/essential_keyboard.rs` (new)

**Purpose**: Add minimal keyboard enhancements for Phase 4

**New Key Handling**:
- Shift+key: Expand current selection
- Escape: Cancel selection
- Tab/Shift+Tab: Navigate between monitors
- Left/Right arrows: Move to adjacent monitor

**Keep Simple**: Avoid complex modifier combinations or advanced shortcuts

---

## Milestone 3: Persistent Configuration System

### Configuration Requirements

#### User-Customizable Settings
- Grid dimensions per monitor (3x2, 4x3, 5x4, etc.)
- Custom hotkey combinations
- Visual appearance preferences (colors, transparency, animations)
- Behavior settings (timeout duration, cross-monitor enabled)

#### Storage Mechanism
- Windows Registry for system-level settings
- JSON configuration files for user preferences  
- Automatic migration between configuration versions

### Implementation Tasks

#### Task 3.1: Configuration Schema
**File**: `src/config/schema.rs` (new)

**Purpose**: Define user-customizable settings structure

**Configuration Areas**:
- Global settings (enabled, hotkey, auto-start)
- Per-monitor settings (grid size, enabled status) using stable `MonitorId`
- Appearance settings (colors, transparency)
- Behavior settings (timeout, cross-monitor policy)

**Features**: Versioning, validation, default fallbacks


```

#### Task 3.2: Configuration Persistence
**File**: `src/config/storage.rs` (new)

**Purpose**: Save and load user configuration

**Storage Options**:
- Registry for system-level settings
- JSON files for user preferences
- Hybrid approach with fallbacks

**Features**: Backup/restore, validation, migration between versions

#### Task 3.3: Configuration Management
**File**: `src/config/manager.rs` (new)

**Purpose**: Coordinate configuration loading, validation, and updates

**Key Features**:
- Load configuration with fallback to defaults
- Save configuration changes
- Notify application components of config changes
- Handle configuration corruption gracefully

**Integration**: Plugs into app controller for runtime configuration updates

#### Task 3.4: Runtime Configuration Updates
**File**: `src/app/config_integration.rs` (new)

**Purpose**: Apply configuration changes without application restart

**Integration Points**:
- Update grid configurations for each monitor
- Change hotkey registration
- Apply appearance changes to overlays
- Validate configuration before applying

**Error Handling**: Graceful rollback on configuration errors

---

## Phase 4A Completion

### Deliverables

After Phase 4A completion, the application will have:
- ✅ Multi-monitor window positioning with stable monitor identification
- ✅ Advanced selection modes with window constraints
- ✅ Persistent user configuration with runtime updates
- ✅ Enhanced keyboard handling for multi-monitor navigation
- ✅ Selection history tracking

### Architecture Impact

Phase 4A adds significant architectural components:

```
src/
├── config/                    # NEW: Configuration management
│   ├── mod.rs
│   ├── schema.rs             # Configuration data structures
│   ├── storage.rs            # Persistence layer
│   └── manager.rs            # Configuration lifecycle
├── platform/
│   ├── multi_monitor.rs      # NEW: Simple multi-monitor layout with MonitorId
│   ├── window_constraints.rs # NEW: Window property detection
│   └── ...existing files...
├── domain/
│   ├── cross_monitor.rs      # NEW: Simple cross-monitor selection (horizontal only)
│   ├── selection_history.rs  # NEW: Selection history tracking
│   └── ...existing files...
├── input/
│   ├── essential_keyboard.rs # NEW: Essential keyboard handling only
│   ├── navigation.rs         # NEW: Simple monitor navigation (left/right)
│   └── ...existing files...
├── app/
│   ├── config_integration.rs # NEW: Config-app integration
│   └── ...existing files...
└── tests/
    └── integration/           # NEW: Integration tests for Phase 4A features
```

**Deferred to Phase 4B**:
- `src/ui/system_tray.rs`
- `src/ui/help_overlay.rs` 
- `src/ui/visual_effects.rs`
- `src/app/user_feedback.rs`
- `scripts/create_portable.ps1`

### Dependency Updates

### Dependencies

**New Dependencies for Phase 4**:
- `serde` + `serde_json` for configuration serialization
- `winreg` for Windows Registry access
- `dirs` for standard directory paths
- `log` + `env_logger` for diagnostic logging
- `thiserror` for structured error handling

**Note**: File watching and build resources deferred to later phases

### Error Handling Strategy

**Approach**: Comprehensive error handling with graceful degradation

**Key Areas**:
- Monitor enumeration failures
- Configuration corruption
- Cross-monitor operation errors
- System integration issues

**User Experience**: Clear error messages, automatic fallbacks, no crashes

---

## Testing Strategy for Phase 4A

### Unit Tests (Per Milestone)
- **Milestone 1**: Monitor coordinate conversion, stable MonitorId generation
- **Milestone 2**: Selection constraint application, history management  
- **Milestone 3**: Configuration serialization, storage round-trips

### Integration Tests
- Cross-monitor selection workflows
- Configuration persistence across restarts
- Multi-monitor hot-plug handling
- Selection history across monitor changes

### Performance Tests  
- Multi-monitor overlay rendering latency
- Configuration load/save performance
- Memory usage with multiple monitors
- Startup time with saved configuration

---

## Critical Implementation Requirements

### Monitor ID Stability (CRITICAL)

**This is the most important requirement for Phase 4 success.**

- **Problem**: Using `usize` indices for monitors breaks with hot-plug, reboot, and configuration changes
- **Solution**: Implement stable `MonitorId` based on Windows device paths or hardware identifiers
- **Implementation**: 
  - Use `GetDisplayConfigBufferSizes` and `QueryDisplayConfig` for stable monitor identification
  - Generate `MonitorId` from device instance path or EDID when available
  - Fall back to resolution + position hash for basic displays
- **Validation**: All monitor references must use `MonitorId`, never indices

### Simple Cross-Monitor Policy (CRITICAL)

**Keep Phase 4 scope limited to horizontal adjacent monitors only.**

- **Phase 4 Scope**: Only `AdjacentHorizontal` policy - left/right monitors only
- **No Vertical**: Defer above/below monitor arrangements to later phases
- **No Complex Layouts**: L-shaped or diagonal arrangements not supported in Phase 4
- **Clear UX**: Users understand they can only span between left/right adjacent monitors
- **Single Desktop Only**: Application operates only on the main Windows desktop

## Risk Assessment & Mitigation

### High-Risk Areas

**Cross-Monitor Coordinate Calculations**
- *Risk*: Incorrect window positioning between monitors with different DPI
- *Mitigation*: Comprehensive test suite with various monitor configurations
- *Fallback*: Single-monitor mode if cross-monitor calculations fail

**Configuration System Complexity**
- *Risk*: Configuration corruption causing application failure
- *Mitigation*: Configuration validation, backup/restore mechanisms
- *Fallback*: Automatic reset to defaults on corruption detection

**System Integration (Tray, Auto-start)**
- *Risk*: Windows security/antivirus false positives
- *Mitigation*: Code signing, gradual rollout, clear documentation
- *Fallback*: Portable mode without system integration

### Medium-Risk Areas

**Advanced Keyboard Handling**
- *Risk*: Conflicts with other applications using similar hotkeys
- *Mitigation*: Configurable hotkeys, conflict detection
- *Fallback*: Graceful degradation to basic functionality

**Performance at Scale**
- *Risk*: Poor performance with many monitors (4+) or large grids
- *Mitigation*: Performance monitoring, optimization, configurable limits
- *Fallback*: Automatic downscaling of features on slower systems

---

## Success Criteria

### Functional Requirements (Phase 4A)
- ✅ Multi-monitor window positioning works correctly with stable MonitorId
- ✅ Configuration persists across application restarts and monitor changes
- ✅ Advanced selection modes (constraints, history) function properly  
- ✅ Cross-monitor selection limited to adjacent horizontal monitors
- ✅ Essential keyboard navigation works between monitors

### Performance Requirements
- ✅ Multi-monitor overlay display latency < 150ms
- ✅ Configuration load time < 200ms
- ✅ Memory usage remains stable with multiple monitors
- ✅ No performance degradation with saved configuration

### Quality Requirements
- ✅ Comprehensive test coverage (>80% for new Phase 4A code)
- ✅ Graceful handling of monitor hot-plug events
- ✅ Configuration corruption handled with automatic fallbacks
- ✅ Clear error messages for cross-monitor constraint violations

---

## Implementation Timeline

### Week 1: Multi-Monitor Foundation (Days 1-3)
- **Days 1-2**: Stable MonitorId system and multi-monitor coordinate handling
- **Day 3**: Cross-monitor selection logic and validation

### Week 2: Advanced Features (Days 4-6)
- **Days 4-5**: Window constraints and selection history
- **Day 6**: Essential keyboard handling enhancements

### Week 3: Configuration System (Days 7-9)
- **Days 7-8**: Configuration schema, storage, and management
- **Day 9**: Runtime configuration updates and integration testing

**Total Phase 4A Duration**: 6-9 days

**Phase 4B Preview**: System tray, help system, visual polish, and production distribution will follow as a separate phase

---

This Phase 4A plan builds the advanced core functionality needed to transform tactile-win from a single-monitor MVP into a sophisticated multi-monitor window management tool with persistent user configuration. Upon completion, users will have a fully functional advanced window manager, with professional polish and distribution features following in Phase 4B.