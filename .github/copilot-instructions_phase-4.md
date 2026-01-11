# Phase 4: Single-Monitor MVP with Configuration

## 🎯 Central Objective
> *"Deliver a production-ready single-monitor window manager with orientation support, window snapping, persistent configuration, and polished UX"*

Phase 4 perfects the single-monitor experience before tackling multi-monitor complexity.

---

## 1. CRITICAL DECISION: Single-Monitor First
**Before expanding to multi-monitor**, master single-monitor:

- **Complete feature set**: Orientation, snapping, configuration
- **Production quality**: Error handling, performance, logging
- **User-friendly**: Configuration UI, validation, feedback
- **Well-tested**: All edge cases covered for single monitor

**Fundamental principle**: Multi-monitor adds significant complexity. Build a solid foundation first.

---

## 2. Monitor Orientation (CRITICAL - Foundation)
**Adaptive grid layout** based on monitor rotation:

**Orientation Detection**:
- Landscape (width > height): 3x2 grid (3 cols, 2 rows)
- Portrait (height > width): 2x3 grid (2 cols, 3 rows)
- Square (rare): 3x3 grid

**Key Requirements**:
- Detect orientation from work area dimensions
- Apply 10% threshold to avoid false positives
- Maintain minimum cell size (480x360) in all orientations
- Reconfigure grid dynamically when orientation changes

---

## 3. Window Snapping (Mental Model)
Before implementing, define what "snapping" means:

**Gap Application**:
- Screen edges: Optional gap (0-50px) from monitor borders
- Between cells: Optional gap between multi-cell selections
- Smart corners: Don't double-apply gaps at intersections
- Configurable: Enable/disable, adjust size per user preference

**User Control**:
```
GapSettings {
    enabled: bool,        // Enable snapping with gaps
    size: u32,           // Gap size in pixels (0-50)
    screen_edges: bool,  // Apply to borders
    between_cells: bool, // Apply to cell boundaries
}
```

---

## 4. Specific Modules

### 4.1 platform::orientation
**Single responsibility**: "Tell me if monitor is landscape or portrait"

**Concrete tasks**:
- Detect orientation from work area dimensions
- Track orientation changes
- Apply threshold to avoid false positives
- **DO NOT** implement grid logic here

### 4.2 domain::snapping
**Single responsibility**: "Apply gaps to a grid-selected rectangle"

**Concrete tasks**:
- Calculate gap-adjusted rectangles
- Detect screen edges vs internal cells
- Handle corner cases (no double gaps)
- Validate minimum window size after gaps
- **Critical**: Pure calculation, no Win32 dependencies

### 4.3 config::schema
**Single responsibility**: "Define and validate all user settings"

**Configuration Areas**:
- Grid settings (dimensions, adaptive orientation)
- Gap settings (enable, size, edge/cell behavior)
- Hotkey configuration
- Visual appearance (colors, transparency, line width)
- Behavior settings (timeout, show help)

### 4.4 config::storage
**Single responsibility**: "Persist configuration to disk"

**Concrete tasks**:
- Save/load JSON configuration from user directory
- Create automatic backups before overwriting
- Validate configuration on load
- Provide default fallbacks for missing/corrupt files
- **File location**: `%APPDATA%\tactile-win\config.json`

### 4.5 config::manager
**Single responsibility**: "Manage runtime configuration state"

**Concrete tasks**:
- Thread-safe configuration access (Arc<RwLock>)
- Update configuration with validation
- Notify components of configuration changes
- Reset to defaults

### 4.6 ui::config_window
**Single responsibility**: "Provide user interface for configuration"

**Concrete tasks**:
- Native Windows dialog or simple window
- Grid size controls with real-time validation
- Gap settings controls
- Hotkey editor
- Save/Cancel/Reset buttons
- Visual feedback for invalid settings

---

## 5. Configuration Workflow

### Initial Setup:
1. Load configuration from disk (or use defaults)
2. Validate all settings
3. Apply to application components
4. Create configuration UI accessible from tray/hotkey

### Runtime Updates:
1. User modifies settings in UI
2. Validate before applying
3. Update in-memory configuration
4. Persist to disk with backup
5. Notify affected components
6. Refresh overlays/grids if needed

### Error Handling:
- Configuration file missing → Use defaults
- Configuration file corrupted → Restore from backup or use defaults
- Invalid settings → Reject with clear error message
- Configuration directory missing → Create automatically

---

## 6. Production Polish

### Error Handling (Comprehensive)
**Error Categories**:
- Fatal: Cannot continue (missing Win32 APIs, critical failures)
- Recoverable: Continue with fallback (config corruption, hotkey conflicts)
- Silent: Log only (transient issues, minor warnings)

**Use `thiserror`** for structured errors with user-friendly messages.

### Performance Optimization
**Critical Areas**:
- Overlay rendering: Cache grids, redraw only on changes
- Configuration: Load once at startup, lazy updates
- Memory: Release resources when idle
- CPU: Minimize message loop overhead

**Performance Targets**:
- Overlay display: < 100ms
- Configuration load: < 100ms
- Memory (idle): < 50MB
- CPU (idle): < 1%

### User Feedback System
**Notification Types**:
- Toast notifications: Non-intrusive status messages
- Error dialogs: Critical errors requiring attention
- Visual feedback: Highlight selected cells, show validation errors

**Example Notifications**:
- "Window positioned successfully"
- "Configuration saved"
- "Invalid grid selection (window non-resizable)"
- "Orientation changed: Grid now 2x3"

### Diagnostic Logging
**Log Strategy**:
- Use `log` crate with `env_logger`
- Log file: `%APPDATA%\tactile-win\logs\tactile-win.log`
- Log rotation: Keep last 5 files, 10MB max each
- Log levels: TRACE, DEBUG, INFO, WARN, ERROR

---

## 7. Implementation Order

### Week 1: Core Features (Days 1-4)
1. **Orientation detection** (platform::orientation)
2. **Adaptive grid logic** (domain::grid extension)
3. **Snapping calculator** (domain::snapping)
4. **Gap configuration** (config::gap_settings)

### Week 2: Configuration (Days 5-8)
5. **Configuration schema** (config::schema)
6. **Configuration storage** (config::storage)
7. **Configuration manager** (config::manager)
8. **App controller integration** (app::controller)

### Week 3: UI and Polish (Days 9-13)
9. **Configuration UI** (ui::config_window)
10. **Validation UI feedback** (real-time validation)
11. **Error handling enhancement** (all modules)
12. **Performance optimization** (ui::renderer, ui::overlay)
13. **User feedback system** (ui::notifications, logging)

**Total Duration**: 8-13 days

---

## 8. Architecture Impact

Phase 4 adds these components:

```
src/
├── config/                    # NEW: Configuration management
│   ├── mod.rs
│   ├── schema.rs             # Complete configuration structure
│   ├── storage.rs            # JSON file persistence
│   └── manager.rs            # Runtime configuration management
├── platform/
│   ├── orientation.rs        # NEW: Monitor orientation detection
│   └── ...existing files...
├── domain/
│   ├── snapping.rs           # NEW: Gap calculation logic
│   └── ...existing files...
├── ui/
│   ├── config_window.rs      # NEW: Configuration UI
│   ├── notifications.rs      # NEW: User feedback system
│   └── ...existing files...
├── logging.rs                # NEW: Diagnostic logging setup
└── tests/
    └── integration/
        ├── orientation.rs    # Orientation detection tests
        ├── snapping.rs       # Snapping logic tests
        └── config.rs         # Configuration persistence tests
```

**Key Points**:
- All new modules are single-monitor focused
- Configuration system is ready for Phase 5 extension
- No multi-monitor logic in this phase

---

## 9. WHAT NOT TO THINK ABOUT YET
🚫 **Phase 5 concerns (avoid these temptations)**:
- Multi-monitor support
- Stable MonitorId system
- Cross-monitor selection
- Per-monitor configuration
- Monitor hot-plug handling
- System tray integration

If you're thinking about multi-monitor, you're getting ahead of yourself.

---

## 10. Phase 4 Exit Checklist
When finished, you should confidently say:
- ✅ Monitor orientation detection works for landscape/portrait/square
- ✅ Adaptive grid layout switches automatically with orientation
- ✅ Window snapping with gaps works and is fully configurable
- ✅ Configuration persists across application restarts
- ✅ Configuration UI is intuitive and validates all inputs
- ✅ Error handling is comprehensive with clear user messages
- ✅ Performance meets all targets
- ✅ Diagnostic logging aids troubleshooting
- ✅ All single-monitor edge cases handled gracefully

---

## Expected Result

A **production-ready single-monitor window manager** where:
1. Users can rotate their monitor and grid adapts automatically
2. Windows snap to grid with configurable visual gaps
3. All settings persist and survive application restarts
4. Configuration UI is simple and intuitive
5. Error handling is robust and informative
6. Performance is smooth and responsive

With this foundation, Phase 5 (multi-monitor) becomes a natural extension rather than a rewrite.

---

## Sources
- Phase 1-3 implementation and lessons learned
- Single-monitor use cases and user feedback
- Windows desktop management best practices
- Configuration system design patterns
