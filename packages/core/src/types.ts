// ============================================
// Window Position Types
// ============================================

// Window positioning (all values in pixels)
// Position defines a bounding box on screen
export interface WindowPosition {
  monitor?: string          // Monitor name or 'primary'
  top?: number              // Pixels from top edge
  bottom?: number           // Pixels from bottom edge
  left?: number             // Pixels from left edge
  right?: number            // Pixels from right edge
  width?: number            // Explicit width (if left+right not specified)
  height?: number           // Explicit height (if top+bottom not specified)
}

// Window configuration
export interface WindowConfig {
  transparent?: boolean     // default: true
  alwaysOnTop?: boolean     // default: true (bar), false (floating)
  resizable?: boolean       // default: false (bar), true (floating)
  decorations?: boolean     // default: false
  skipTaskbar?: boolean     // default: true
  clickThrough?: boolean    // Ignore mouse events (for overlays)
}

// ============================================
// Global Config Types
// ============================================

// Theme configuration
export interface ThemeConfig {
  mode: 'light' | 'dark' | 'system'
  accentColor?: string
}

// Global settings
export interface GlobalSettings {
  hotReload: boolean
  devMode: boolean
}

// Secrets configuration
export interface SecretsConfig {
  github?: { token: string }
}

// UI configuration for loading user-built frontends
export interface UiConfig {
  /** Custom path to UI dist folder (supports ~ expansion) */
  distPath?: string
}

// Global Fluopanel configuration (fluopanel.json schema)
export interface FluopanelConfig {
  version: number           // Schema version (2)
  theme: ThemeConfig
  settings: GlobalSettings
  secrets?: SecretsConfig
  ui?: UiConfig
}

// ============================================
// Monitor Types
// ============================================

// Monitor information
export interface MonitorInfo {
  name: string
  width: number
  height: number
  x: number
  y: number
  scaleFactor: number
}

// ============================================
// Inline Window Types (for <Window> component)
// ============================================

/** Options for creating an inline window */
export interface InlineWindowOptions {
  /** Unique window ID (used as window label) */
  id: string
  /** CSS-like positioning */
  position: WindowPosition
  /** Window configuration */
  window?: WindowConfig
  /** URL to load (defaults to current page with ?window=id) */
  url?: string
}

/** Runtime context passed to window components */
export interface WindowContext {
  /** Window ID */
  id: string
  /** Window label (for IPC) */
  label: string
  /** Whether running as coordinator (parent) or window (child window) */
  mode: 'coordinator' | 'window'
}

/** Window registration for coordinator mode */
export interface WindowRegistration {
  id: string
  component: string  // Component name for dynamic import
  position: WindowPosition
  window?: WindowConfig
}

// ============================================
// Popover Types
// ============================================

/** Popover alignment relative to anchor element */
export type PopoverAlign = 'start' | 'center' | 'end'

/** Popover anchor position (from trigger element's getBoundingClientRect) */
export interface PopoverAnchor {
  x: number
  y: number
  width: number
  height: number
}

/** Options for opening a popover window */
export interface PopoverOpenOptions {
  /** Unique popover ID */
  id: string
  /** Anchor element position */
  anchor: PopoverAnchor
  /** Initial popover width in pixels (default: 300, auto-resized by content) */
  width?: number
  /** Initial popover height in pixels (default: 200, auto-resized by content) */
  height?: number
  /** Alignment relative to anchor (default: 'center') */
  align?: PopoverAlign
  /** Vertical offset from anchor (default: 8) */
  offsetY?: number
}

/** Popover info returned after open/toggle */
export interface PopoverInfo {
  id: string
  label: string
  /** Whether the popover was closed (toggle mode) */
  closed: boolean
  /** Maximum available height for the popover (from anchor bottom to screen bottom) */
  maxHeight: number
}

/** Popover context for determining current window type */
export interface PopoverContext {
  /** Popover ID (from URL parameter) */
  id: string | null
  /** Whether current window is a popover */
  isPopover: boolean
  /** Maximum available height for the popover (from URL parameter) */
  maxHeight: number | null
}
