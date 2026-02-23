//! Bindable action enum and metadata.

// =============================================================================
// Bindable Action enum
// =============================================================================

/// Every action that can be bound to a key.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum BindableAction {
    // Camera
    CameraPanUp,
    CameraPanDown,
    CameraPanLeft,
    CameraPanRight,
    CameraRotateLeft,
    CameraRotateRight,
    CameraZoomIn,
    CameraZoomOut,

    // Simulation speed
    TogglePause,
    SpeedNormal,
    SpeedFast,
    SpeedFastest,

    // Tools
    ToolRoad,
    ToolZoneRes,
    ToolZoneCom,
    ToolBulldoze,
    ToolInspect,
    ToggleGridSnap,
    ToggleCurveDraw,
    DeleteBuilding,
    Escape,

    // Overlays
    OverlayCycleNext,

    // Panels
    ToggleJournal,
    ToggleCharts,
    ToggleAdvisor,
    TogglePolicies,
    ToggleSettings,
    ToggleSearch,

    // Save/Load (ctrl-modified)
    QuickSave,
    QuickLoad,
    NewGame,

    // Screenshot
    Screenshot,
}

impl BindableAction {
    /// Human-readable label for display in the settings UI.
    pub fn label(self) -> &'static str {
        match self {
            Self::CameraPanUp => "Camera Pan Up",
            Self::CameraPanDown => "Camera Pan Down",
            Self::CameraPanLeft => "Camera Pan Left",
            Self::CameraPanRight => "Camera Pan Right",
            Self::CameraRotateLeft => "Camera Rotate Left",
            Self::CameraRotateRight => "Camera Rotate Right",
            Self::CameraZoomIn => "Camera Zoom In",
            Self::CameraZoomOut => "Camera Zoom Out",
            Self::TogglePause => "Toggle Pause",
            Self::SpeedNormal => "Speed: Normal (1x)",
            Self::SpeedFast => "Speed: Fast (2x)",
            Self::SpeedFastest => "Speed: Fastest (4x)",
            Self::ToolRoad => "Tool: Road",
            Self::ToolZoneRes => "Tool: Zone Residential",
            Self::ToolZoneCom => "Tool: Zone Commercial",
            Self::ToolBulldoze => "Tool: Bulldoze",
            Self::ToolInspect => "Tool: Inspect",
            Self::ToggleGridSnap => "Toggle Grid Snap",
            Self::ToggleCurveDraw => "Toggle Curve Draw",
            Self::DeleteBuilding => "Delete Building",
            Self::Escape => "Cancel / Deselect",
            Self::OverlayCycleNext => "Cycle Overlay",
            Self::ToggleJournal => "Toggle Journal",
            Self::ToggleCharts => "Toggle Charts",
            Self::ToggleAdvisor => "Toggle Advisor",
            Self::TogglePolicies => "Toggle Policies",
            Self::ToggleSettings => "Toggle Settings",
            Self::ToggleSearch => "Toggle Search",
            Self::QuickSave => "Quick Save",
            Self::QuickLoad => "Quick Load",
            Self::NewGame => "New Game",
            Self::Screenshot => "Screenshot",
        }
    }

    /// Category for grouping in the settings UI.
    pub fn category(self) -> &'static str {
        match self {
            Self::CameraPanUp
            | Self::CameraPanDown
            | Self::CameraPanLeft
            | Self::CameraPanRight
            | Self::CameraRotateLeft
            | Self::CameraRotateRight
            | Self::CameraZoomIn
            | Self::CameraZoomOut => "Camera",

            Self::TogglePause | Self::SpeedNormal | Self::SpeedFast | Self::SpeedFastest => "Speed",

            Self::ToolRoad
            | Self::ToolZoneRes
            | Self::ToolZoneCom
            | Self::ToolBulldoze
            | Self::ToolInspect
            | Self::ToggleGridSnap
            | Self::ToggleCurveDraw
            | Self::DeleteBuilding
            | Self::Escape => "Tools",

            Self::OverlayCycleNext => "Overlays",

            Self::ToggleJournal
            | Self::ToggleCharts
            | Self::ToggleAdvisor
            | Self::TogglePolicies
            | Self::ToggleSettings
            | Self::ToggleSearch => "Panels",

            Self::QuickSave | Self::QuickLoad | Self::NewGame | Self::Screenshot => "System",
        }
    }

    /// All bindable actions in display order.
    pub const ALL: &'static [BindableAction] = &[
        Self::CameraPanUp,
        Self::CameraPanDown,
        Self::CameraPanLeft,
        Self::CameraPanRight,
        Self::CameraRotateLeft,
        Self::CameraRotateRight,
        Self::CameraZoomIn,
        Self::CameraZoomOut,
        Self::TogglePause,
        Self::SpeedNormal,
        Self::SpeedFast,
        Self::SpeedFastest,
        Self::ToolRoad,
        Self::ToolZoneRes,
        Self::ToolZoneCom,
        Self::ToolBulldoze,
        Self::ToolInspect,
        Self::ToggleGridSnap,
        Self::ToggleCurveDraw,
        Self::DeleteBuilding,
        Self::Escape,
        Self::OverlayCycleNext,
        Self::ToggleJournal,
        Self::ToggleCharts,
        Self::ToggleAdvisor,
        Self::TogglePolicies,
        Self::ToggleSettings,
        Self::ToggleSearch,
        Self::QuickSave,
        Self::QuickLoad,
        Self::NewGame,
        Self::Screenshot,
    ];
}
