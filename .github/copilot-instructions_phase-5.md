# Phase 5: Multi-Monitor Support and Production Distribution

## 🎯 Central Objective
> *"Extend the single-monitor MVP to full multi-monitor support with stable monitor identification, per-monitor configuration, system integration, and production-ready distribution"*

Phase 5 completes the application with multi-monitor capabilities and professional polish.

---

## 1. CRITICAL DECISION: Stable Monitor Identification
**Before implementing multi-monitor**, solve the identity problem:

- **Problem**: Monitor indices change with hot-plug, reboots, and display settings
- **Solution**: Stable `MonitorId` based on hardware identifiers
- **Fallback**: Resolution + position hash for generic displays
- **Persistence**: Configuration survives monitor reconnection

**Fundamental principle**: Configurations must persist across monitor changes.

---

## 2. Monitor Identity (CRITICAL - Foundation)
**Stable MonitorId** using Windows device information:

**ID Generation Strategy**:
- Primary: Device instance path from `QueryDisplayConfig`
- Secondary: EDID data (manufacturer + product + serial)
- Fallback: Hash of resolution + position
- Never: Simple indices (0, 1, 2)

**Key Requirements**:
- Same ID after monitor reconnection
- Same ID after system reboot
- Different IDs for different monitors
- Handle monitors without EDID (generic displays)

---

## 3. Multi-Monitor Layout (Mental Model)
Before implementing, define monitor relationships:

**Spatial Relationships**:
```
MonitorRelation:
- Left/Right: Horizontally adjacent (Phase 5 focus)
- Above/Below: Vertically adjacent (future)
- Diagonal: Not adjacent (future)
```

**Layout Management**:
```
MultiMonitorLayout {
    monitors: HashMap<MonitorId, Monitor>,
    adjacency: HashMap<MonitorId, Vec<(MonitorId, Relation)>>,
    primary: MonitorId,
}
```

**Phase 5 Policy**: Support only horizontally adjacent monitors (left/right).

---

## 4. Specific Modules

### 4.1 platform::monitor_id
**Single responsibility**: "Generate stable, persistent monitor identifiers"

**Concrete tasks**:
- Extract device path from Windows APIs
- Parse EDID for manufacturer/product/serial
- Generate fallback hash for generic displays
- Serialize/deserialize for configuration storage
- **DO NOT** implement layout logic here

### 4.2 platform::multi_monitor_layout
**Single responsibility**: "Describe spatial relationships between monitors"

**Concrete tasks**:
- Detect monitor adjacency (left/right/above/below)
- Build adjacency graph for navigation
- Find monitors by relation (e.g., "left of primary")
- Validate cross-monitor selections
- **Critical**: Phase 5 supports only horizontal adjacency

### 4.3 domain::cross_monitor_selection
**Single responsibility**: "Handle window selections spanning monitors"

**Concrete tasks**:
- Track selection start/end across different monitors
- Validate selections according to policy (adjacent only)
- Calculate combined rectangle for cross-monitor spans
- Handle DPI differences when combining rectangles
- **Policy enforcement**: Reject non-adjacent selections

### 4.4 config::per_monitor
**Single responsibility**: "Manage configuration for each monitor independently"

**Configuration Structure**:
```
AppConfig {
    global: GlobalConfig,              // Hotkey, cross-monitor policy
    monitors: HashMap<MonitorId, MonitorConfig>,  // Per-monitor settings
    appearance: AppearanceConfig,      // Global visual settings
    behavior: BehaviorConfig,          // Global behavior
}

MonitorConfig {
    grid: GridConfig,      // Dimensions, adaptive orientation
    gaps: GapSettings,     // Snapping configuration
    enabled: bool,         // Enable this monitor
}
```

### 4.5 ui::multi_monitor_config
**Single responsibility**: "Visual interface for multi-monitor configuration"

**UI Components**:
- Visual monitor layout (drag-drop style diagram)
- Per-monitor settings panel
- Global cross-monitor settings
- Real-time validation feedback
- **Interaction**: Click monitor in diagram to configure it

### 4.6 ui::system_tray
**Single responsibility**: "System tray integration for background operation"

**Concrete tasks**:
- Create tray icon with context menu
- Handle tray icon clicks (left = activate, right = menu)
- Context menu: Settings, Help, Exit
- Toast notifications integration
- **DO NOT** implement full application logic here

### 4.7 platform::auto_start
**Single responsibility**: "Windows auto-start integration"

**Concrete tasks**:
- Register in Windows Registry Run key
- Enable/disable auto-start from configuration
- Verify auto-start status
- Clean removal on uninstall

---

## 5. Multi-Monitor Workflow

### Monitor Detection and Configuration:
1. Enumerate monitors with stable IDs
2. Load per-monitor configurations (or use defaults)
3. Build spatial layout (adjacency relationships)
4. Create grids for each monitor
5. Apply saved settings to each monitor

### Cross-Monitor Selection:
1. User activates overlay (all monitors show grids)
2. User navigates between monitors (arrow keys)
3. User selects start cell on monitor A
4. User navigates to monitor B (if desired)
5. User selects end cell on monitor B
6. Validate: Are monitors adjacent? (Policy check)
7. Calculate combined rectangle with DPI awareness
8. Position window across monitors

### Hot-Plug Handling:
1. Detect monitor configuration change
2. Identify added/removed/updated monitors
3. Load saved configuration for added monitors
4. Preserve configuration for removed monitors
5. Update grids and overlays
6. Notify user of changes (optional)

---

## 6. System Integration

### System Tray
**Context Menu Items**:
- Configure Settings
- Help / Documentation
- About
- Exit

**Tray Icon States**:
- Normal: Ready
- Active: Selection mode active
- Error: Configuration issue

### Auto-Start with Windows
**Registry Key**: `HKEY_CURRENT_USER\Software\Microsoft\Windows\CurrentVersion\Run`

**Considerations**:
- User opt-in (not forced)
- Clean uninstall removes entry
- Respect Windows security policies

### Windows Notifications
**Use Native APIs**:
- Toast notifications for status updates
- Action center integration (Windows 10+)
- Fallback to balloon tips (older Windows)

**Notification Examples**:
- "Monitor configuration changed: 2 displays detected"
- "Configuration saved successfully"
- "Update available: Version 1.1.0"

---

## 7. Help System and Documentation

### In-App Help
**Help Overlay** (F1 key):
- Getting Started
- Grid Selection Basics
- Multi-Monitor Usage
- Keyboard Shortcuts
- Troubleshooting

**Interactive Tutorial** (first run):
- Welcome screen
- Hotkey activation demo
- Single selection demo
- Multi-cell selection demo
- Configuration overview

### External Documentation
**User Guide** (Markdown/HTML):
1. Installation and Setup
2. Basic Usage
3. Configuration Options
4. Multi-Monitor Support
5. Keyboard Shortcuts Reference
6. FAQ
7. Troubleshooting

---

## 8. Production Distribution

### Windows Installer
**Installer Features** (NSIS or WiX):
- Install application binary and assets
- Create start menu shortcuts
- Optional desktop shortcut
- Optional auto-start configuration
- Install prerequisites (VC++ runtime if needed)
- Clean uninstall (files + registry)

### Auto-Update System
**Update Workflow**:
1. Check GitHub releases on startup
2. Compare current version with latest
3. Notify user of available update
4. Download installer in background
5. Prompt user to install
6. Run installer and exit current instance

**Version Management**:
- Semantic versioning (1.0.0, 1.1.0, 2.0.0)
- Git hash in build metadata
- Version display in About dialog

### CI/CD Pipeline
**GitHub Actions Workflow**:
- Trigger on version tags (v1.0.0)
- Build release binary (cargo build --release)
- Run tests (cargo test --release)
- Build installer (NSIS/WiX)
- Create GitHub Release with artifacts
- Attach release notes

---

## 9. Implementation Order

### Week 1: Stable Monitor IDs (Days 1-3)
1. **MonitorId system** (platform::monitor_id)
2. **Monitor enumeration update** (use MonitorId everywhere)
3. **Configuration migration** (Phase 4 → Phase 5 format)

### Week 2: Multi-Monitor Core (Days 4-7)
4. **Multi-monitor layout** (platform::multi_monitor_layout)
5. **Cross-monitor selection** (domain::cross_monitor_selection)
6. **Multi-monitor positioning** (platform::window extension)
7. **Monitor navigation** (input::navigation)

### Week 3: Per-Monitor Config (Days 8-10)
8. **Per-monitor configuration** (config::per_monitor)
9. **Multi-monitor config UI** (ui::multi_monitor_config)
10. **Hot-plug handling** (app::controller extension)

### Week 4: System Integration (Days 11-13)
11. **System tray** (ui::system_tray)
12. **Auto-start** (platform::auto_start)
13. **Windows notifications** (ui::toast_notifications)

### Week 5: Help and Distribution (Days 14-18)
14. **Help overlay** (ui::help_overlay)
15. **Interactive tutorial** (ui::tutorial)
16. **User documentation** (docs/)
17. **Windows installer** (installer/setup.nsi)
18. **Auto-update system** (update::auto_update)
19. **CI/CD pipeline** (.github/workflows/release.yml)

**Total Duration**: 12-18 days

---

## 10. Architecture Impact

Phase 5 adds these components:

```
src/
├── config/
│   ├── per_monitor.rs        # NEW: Per-monitor configuration
│   ├── migration.rs          # NEW: Config version migration
│   └── ...existing files...
├── platform/
│   ├── monitor_id.rs         # NEW: Stable monitor identifiers
│   ├── multi_monitor_layout.rs # NEW: Monitor spatial relationships
│   ├── auto_start.rs         # NEW: Windows auto-start integration
│   └── ...existing files...
├── domain/
│   ├── cross_monitor_selection.rs # NEW: Multi-monitor selection logic
│   └── ...existing files...
├── input/
│   ├── navigation.rs         # NEW: Monitor navigation (arrow keys)
│   └── ...existing files...
├── ui/
│   ├── system_tray.rs        # NEW: System tray integration
│   ├── multi_monitor_config.rs # NEW: Multi-monitor config UI
│   ├── monitor_layout_view.rs # NEW: Visual monitor layout widget
│   ├── help_overlay.rs       # NEW: In-app help system
│   ├── tutorial.rs           # NEW: First-run tutorial
│   └── toast_notifications.rs # NEW: Windows toast notifications
├── update/                   # NEW: Auto-update system
│   ├── mod.rs
│   └── auto_update.rs
└── tests/
    └── integration/
        ├── multi_monitor.rs  # Multi-monitor workflow tests
        ├── hot_plug.rs       # Hot-plug handling tests
        └── monitor_id.rs     # MonitorId stability tests

docs/                          # NEW: User documentation
├── user-guide.md
├── faq.md
└── troubleshooting.md

installer/                     # NEW: Windows installer
├── setup.nsi                 # NSIS installer script
└── assets/
    └── icon.ico

.github/workflows/             # NEW: CI/CD
└── release.yml               # Automated release pipeline
```

**Key Points**:
- Multi-monitor support built on Phase 4 foundation
- MonitorId system is critical - extensive testing required
- System integration (tray, auto-start) adds OS dependencies
- Distribution artifacts (installer, updater) complete the product

---

## 11. CRITICAL WARNINGS

### Cross-Monitor Complexity
**This is significantly harder than single-monitor**:
- Virtual coordinate system (secondary monitors can be at negative coords)
- Per-monitor DPI scaling differences
- Monitor arrangement detection (left/right/above/below)
- Work area variations (taskbar position per monitor)
- Window focus complications when moving between monitors

**Mitigation**: Implement Phase 4 thoroughly first. Test extensively on real hardware.

### MonitorId Stability Risk
**If MonitorIds aren't stable, everything breaks**:
- Lost configuration after monitor reconnect
- Lost configuration after system reboot
- Wrong configuration applied to wrong monitor

**Mitigation**: Multiple fallback strategies, extensive testing across hardware configurations.

---

## 12. Phase 5 Exit Checklist
When finished, you should confidently say:
- ✅ MonitorId system generates stable identifiers across reboots
- ✅ Multi-monitor layout detects adjacency relationships correctly
- ✅ Cross-monitor selection works for horizontally adjacent monitors
- ✅ Per-monitor configuration persists and applies correctly
- ✅ Hot-plug handling updates configuration without data loss
- ✅ Multi-monitor config UI is intuitive with visual layout
- ✅ System tray integration works on Windows 10 and 11
- ✅ Auto-start registers correctly and removes cleanly
- ✅ Help system provides comprehensive guidance
- ✅ Installer/uninstaller work cleanly on all supported Windows versions
- ✅ Auto-update system checks and installs updates successfully
- ✅ CI/CD pipeline builds and publishes releases automatically

---

## Expected Result

A **complete, production-ready multi-monitor window manager** where:
1. Users with multiple monitors can position windows across displays
2. Configuration persists correctly even when monitors are reconnected
3. Each monitor can have independent grid configuration
4. Application runs in background with system tray integration
5. Auto-start works reliably on Windows 10 and 11
6. Help system guides users through all features
7. Updates install automatically with user consent
8. Professional installer handles all setup tasks

The application is ready for public release and daily use.

---

## Sources
- Multi-monitor Windows development best practices
- Windows system integration guidelines
- Application distribution and update patterns
- Phase 1-4 implementation experience
