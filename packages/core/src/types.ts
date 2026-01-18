// Widget positioning (all values in pixels)
// Position defines a bounding box on screen
export interface WidgetPosition {
  monitor?: string          // Monitor name or 'primary'
  top?: number              // Pixels from top edge
  bottom?: number           // Pixels from bottom edge
  left?: number             // Pixels from left edge
  right?: number            // Pixels from right edge
  width?: number            // Explicit width (if left+right not specified)
  height?: number           // Explicit height (if top+bottom not specified)
}

// Widget window configuration
export interface WidgetWindowConfig {
  transparent?: boolean     // default: true
  alwaysOnTop?: boolean     // default: true (bar), false (floating)
  resizable?: boolean       // default: false (bar), true (floating)
  decorations?: boolean     // default: false
  skipTaskbar?: boolean     // default: true
  clickThrough?: boolean    // Ignore mouse events (for overlays)
}

// Widget manifest (widget.json schema)
export interface WidgetManifest {
  id: string                // Unique identifier
  name: string              // Display name
  version: string           // Semantic version

  type: 'bar' | 'floating'  // Widget type

  position: WidgetPosition
  window?: WidgetWindowConfig

  entry: string             // Entry point (default: 'index.html')
  devUrl?: string           // Development server URL
}

// Widget instance (loaded widget)
export interface WidgetInstance {
  widgetId: string          // References WidgetManifest.id
  instanceId: string        // Unique instance ID (for duplicates)
  enabled: boolean
  overrides?: Partial<WidgetPosition>
}

// Monitor information
export interface MonitorInfo {
  name: string
  width: number
  height: number
  x: number
  y: number
  scaleFactor: number
}

// Configuration types
export interface BarConfig {
  position: 'top' | 'bottom'
  height: number
  opacity: number
}

export interface WidgetConfig {
  enabled: boolean
  [key: string]: unknown
}

export interface GitHubConfig {
  token?: string
}

export interface AppConfig {
  bar: BarConfig
  widgets: {
    workspaces: WidgetConfig
    clock: WidgetConfig & { format?: string }
    battery: WidgetConfig
    cpu: WidgetConfig
    memory: WidgetConfig
    network: WidgetConfig
  }
  theme: {
    mode: 'light' | 'dark' | 'system'
  }
  github?: GitHubConfig
}

// Global Arcana configuration
export interface ArcanaConfig {
  widgets: WidgetInstance[]
  settings: {
    hotReload: boolean
    devMode: boolean
  }
  theme: {
    mode: 'light' | 'dark' | 'system'
    accentColor?: string
  }
}

// ============================================
// Inline Widget Types (for <Widget> component)
// ============================================

/** Options for creating an inline widget window */
export interface InlineWidgetOptions {
  /** Unique widget ID (used as window label) */
  id: string
  /** CSS-like positioning */
  position: WidgetPosition
  /** Window configuration */
  window?: WidgetWindowConfig
  /** URL to load (defaults to current page with ?widget=id) */
  url?: string
}

/** Runtime context passed to widget components */
export interface WidgetContext {
  /** Widget ID */
  id: string
  /** Window label (for IPC) */
  label: string
  /** Whether running as coordinator (parent) or widget (child window) */
  mode: 'coordinator' | 'widget'
}

/** Widget registration for coordinator mode */
export interface WidgetRegistration {
  id: string
  component: string  // Component name for dynamic import
  position: WidgetPosition
  window?: WidgetWindowConfig
}

// ============================================
// Popup Types
// ============================================

/** Popup alignment relative to anchor element */
export type PopupAlign = 'start' | 'center' | 'end'

/** Popup anchor position (from trigger element's getBoundingClientRect) */
export interface PopupAnchor {
  x: number
  y: number
  width: number
  height: number
}

/** Options for opening a popup window */
export interface PopupOpenOptions {
  /** Unique popup ID */
  id: string
  /** Anchor element position */
  anchor: PopupAnchor
  /** Popup width in pixels */
  width: number
  /** Popup height in pixels */
  height: number
  /** Alignment relative to anchor (default: 'center') */
  align?: PopupAlign
  /** Vertical offset from anchor (default: 8) */
  offsetY?: number
}

/** Popup info returned after creation */
export interface PopupInfo {
  id: string
  label: string
}

/** Popup context for determining current window type */
export interface PopupContext {
  /** Popup ID (from URL parameter) */
  id: string | null
  /** Whether current window is a popup */
  isPopup: boolean
}
