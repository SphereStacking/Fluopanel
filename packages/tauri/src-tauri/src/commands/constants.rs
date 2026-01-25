/// UI layout constants for window and popover positioning
pub mod geometry {
    /// Shadow padding for window sizing (p-20 = 80px each side = 160px total)
    pub const SHADOW_PADDING: f64 = 160.0;

    /// Top margin for menu bar area
    pub const TOP_MARGIN: f64 = 80.0;

    /// Default vertical offset for popovers below their anchor
    pub const DEFAULT_POPOVER_OFFSET_Y: f64 = 8.0;

    /// Minimum available height for popovers
    pub const MIN_AVAILABLE_HEIGHT: f64 = 100.0;
}
