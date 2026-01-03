---
papersize: legal
geometry: margin=1.5cm
fontsize: 12pt
output: pdf_document
header-includes:
 - \pagenumbering{gobble}
---

# Phase 4: Advanced Features & Cross-Monitor Support

**Status**: Phase 3 Complete ✓  
**Prerequisite**: Working single-monitor MVP with keyboard capture and window positioning  
**Goal**: Advanced window management features and cross-monitor functionality

---

## Phase 4 Overview

Phase 4 transforms the single-monitor MVP into a comprehensive window management solution with advanced features, cross-monitor support, persistent configuration, and professional user experience enhancements.

### Current Implementation Status

After Phase 3 completion:
- ✅ Single-monitor grid selection and positioning
- ✅ Global hotkey activation (Ctrl+Alt+T)
- ✅ Modal keyboard capture with low-level hooks
- ✅ Visual grid overlays with tiny-skia rendering
- ✅ DPI-aware coordinate handling
- ✅ RAII resource management throughout

### Phase 4 Objectives

1. **Multi-Monitor Window Management**: Enable window positioning across multiple physical monitors within the main desktop
2. **Persistent Configuration**: User-customizable grids, hotkeys, and preferences with registry/file storage
3. **Enhanced User Experience**: System tray integration, on-screen help, visual feedback improvements
4. **Production Polish**: Comprehensive error handling, logging, portable distribution
5. **Simple Cross-Monitor Features**: Basic window movement between adjacent monitors only

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

### Milestone 4: Enhanced User Experience (Low-Medium Complexity)
**Estimated Effort**: 2-3 days  
**Focus**: System tray, help system, visual polish

### Milestone 5: Production Readiness (Low Complexity)
**Estimated Effort**: 1-2 days  
**Focus**: Installer, auto-start, final testing

---

## Milestone 1: Cross-Monitor Foundation

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

## Milestone 4: Enhanced User Experience

### System Integration Features

#### System Tray Integration  
- Persistent system tray icon with context menu
- Quick enable/disable toggle
- Recent selections access
- Configuration shortcut

#### On-Screen Help System
- Overlay help during selection mode
- Keyboard shortcut reference
- Tutorial mode for first-time users

#### Visual Polish
- Smooth animations for overlay transitions
- High-contrast mode support
- Custom color schemes
- Selection preview

### Implementation Tasks

#### Task 4.1: System Tray Integration
**File**: `src/ui/system_tray.rs` (new)

**Purpose**: Persistent system tray presence with quick access menu

**Features**:
- System tray icon with context menu
- Quick enable/disable toggle
- Access to configuration
- Application exit option
- Status indication (enabled/disabled)

#### Task 4.2: Help System
**File**: `src/ui/help_overlay.rs` (new)

**Purpose**: On-screen help and tutorials for new users

**Help Types**:
- Quick keyboard reference overlay
- First-time user tutorial
- Context-sensitive tips

**Display**: Overlay help content on grid during selection mode

#### Task 4.3: Visual Improvements
**File**: `src/ui/visual_effects.rs` (new)

**Purpose**: Basic visual polish and accessibility improvements

**Improvements**:
- High-contrast mode support
- Customizable color schemes
- Selection preview effects
- Smooth overlay transitions

**Scope**: Keep animations simple - focus on usability over complexity

#### Task 4.4: Error Handling & User Feedback
**File**: `src/app/user_feedback.rs` (new)

**Purpose**: Comprehensive error handling with user-friendly feedback

**Features**:
- Clear error messages for failed operations
- Status notifications for successful actions
- Graceful degradation on system errors
- Diagnostic logging for troubleshooting

**Priority**: Focus on cross-monitor selection errors and configuration issues

---

## Milestone 5: Production Readiness

### Release Preparation

#### Installer Package
- Windows MSI installer with proper registration
- Auto-start configuration option
- Uninstaller with complete cleanup

#### Quality Assurance  
- Comprehensive test suite including integration tests
- Performance benchmarks and optimization
- Memory leak detection and resource cleanup validation

#### Documentation & Distribution
- User manual and troubleshooting guide
- GitHub releases with changelog
- Code signing for security

### Implementation Tasks

#### Task 5.1: Portable Distribution
**File**: `scripts/create_portable.ps1` (new)

**Purpose**: Create simple portable distribution package for Phase 4

**Package Contents**:
- Release executable
- Documentation files (README, LICENSE)
- Optional auto-start batch file
- Zip package for distribution

**Approach**: Keep simple - MSI installer deferred to later phase

#### Task 5.2: Integration Testing
**File**: `tests/integration/` (new)

**Purpose**: Validate complete Phase 4 workflows

**Test Scenarios**:
- Complete selection workflow from hotkey to window positioning
- Multi-monitor configuration handling
- Configuration persistence across restarts
- Resource cleanup verification

**Focus**: End-to-end user scenarios rather than unit test details

#### Task 5.3: Release Validation
**File**: `src/app/release_validation.rs` (new)

**Purpose**: Basic validation before release

**Validation Areas**:
- System compatibility checks
- Resource lifecycle verification
- Configuration system validation
- Core functionality smoke tests

**Keep Simple**: No complex performance monitoring - focus on correctness

#### Task 5.4: Build Automation
**File**: `.github/workflows/release.yml` (new)

**Purpose**: Automated build and release process

**Workflow Steps**:
- Trigger on version tags
- Build release binary
- Create portable package
- Upload artifacts
- Create GitHub release

**Keep Simple**: No code signing or complex MSI building in Phase 4

---

## Architecture Integration

### Module Structure Changes

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
├── ui/
│   ├── system_tray.rs        # NEW: System tray integration
│   ├── help_overlay.rs       # NEW: Help system
│   ├── visual_effects.rs     # NEW: Basic visual improvements
│   └── ...existing files...
├── app/
│   ├── config_integration.rs # NEW: Config-app integration
│   ├── user_feedback.rs      # NEW: Error handling & feedback
│   ├── release_validation.rs # NEW: Basic release checks
│   └── ...existing files...
└── scripts/                   # NEW: Build and distribution scripts
    └── create_portable.ps1
```

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

## Testing Strategy for Phase 4

### Unit Tests (Per Milestone)
- **Milestone 1**: Virtual coordinate conversion, monitor arrangement detection
- **Milestone 2**: Selection constraint application, history management  
- **Milestone 3**: Configuration serialization, storage round-trips
- **Milestone 4**: Visual effect calculations, tray menu logic
- **Milestone 5**: Performance benchmarks, resource cleanup

### Integration Tests
- Cross-monitor selection workflows
- Configuration persistence across restarts
- System tray interaction scenarios
- Help system navigation
- Multi-monitor hot-plug handling

### Performance Tests  
- Overlay rendering latency (target: <16ms for 60fps)
- Memory usage during extended operation (target: <50MB)
- Startup time (target: <2 seconds)
- Configuration load time (target: <100ms)

### Accessibility Tests
- High-contrast mode functionality
- Screen reader compatibility (where applicable)
- Keyboard-only navigation
- Large DPI scaling (200%+) support

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

### Functional Requirements
- ✅ Cross-monitor window positioning works correctly
- ✅ Configuration persists across application restarts  
- ✅ System tray integration provides quick access to features
- ✅ Help system enables new users to learn the application
- ✅ Application installs and uninstalls cleanly

### Performance Requirements  
- ✅ Overlay display latency < 100ms on modern hardware
- ✅ Memory usage remains stable during extended operation
- ✅ Application startup time < 3 seconds
- ✅ No detectable impact on system performance when idle

### Quality Requirements
- ✅ Comprehensive test coverage (>80% for new Phase 4 code)
- ✅ No memory leaks detected in 24-hour stress testing
- ✅ Graceful error handling with user-friendly feedback
- ✅ Professional installer with proper Windows integration

---

## Implementation Timeline

### Week 1: Foundation
- **Days 1-2**: Virtual desktop coordinate system
- **Days 3-4**: Cross-monitor selection logic
- **Day 5**: Integration and initial testing

### Week 2: Advanced Features  
- **Days 1-2**: Advanced selection modes and constraints
- **Days 3-4**: Configuration system implementation
- **Day 5**: Configuration integration and testing

### Week 3: User Experience
- **Days 1-2**: System tray and help system
- **Days 3-4**: Visual effects and polish
- **Day 5**: User experience testing and refinement

### Week 4: Production Ready
- **Days 1-2**: Installer and distribution
- **Days 3-4**: Performance optimization and testing  
- **Day 5**: Final testing, documentation, release preparation

---

This comprehensive Phase 4 plan transforms the tactile-win application from a functional MVP into a production-ready, professional window management solution with advanced features and excellent user experience.