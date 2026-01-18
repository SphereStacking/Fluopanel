// Provider interface
export interface Provider<T> {
  get(): Promise<T>
  subscribe(callback: (data: T) => void): () => void  // returns unsubscribe function
}

// Workspace related types
export interface Workspace {
  id: string
  displayName?: string
  focused: boolean
  visible: boolean
  windows: Window[]
  monitor: number
}

export interface Window {
  id: number
  app: string
  title: string
  focused: boolean
}

export interface AppIcon {
  app: string
  icon: string | null  // base64-encoded PNG
}

// System information types
export interface BatteryInfo {
  percent: number
  charging: boolean
  timeToEmpty?: number  // minutes
  timeToFull?: number   // minutes
}

export interface CpuInfo {
  usage: number         // 0-100
  temperature?: number  // Celsius
}

export interface MemoryInfo {
  total: number         // bytes
  used: number          // bytes
  usage: number         // 0-100
}

export interface NetworkInfo {
  interface: string
  type: 'wifi' | 'ethernet' | 'unknown'
  ssid?: string
  signalStrength?: number  // 0-100
  connected: boolean
}

export interface DateInfo {
  timestamp: number
  formatted: string
}

export interface MediaInfo {
  playing: boolean
  title?: string
  artist?: string
  album?: string
  duration?: number       // seconds
  position?: number       // seconds
  app?: string            // Spotify, Music, etc.
  artworkUrl?: string     // Album art URL or base64
}

export interface VolumeInfo {
  volume: number          // 0-100
  muted: boolean
  outputDevice?: string   // Speaker name
}

export interface ActiveAppInfo {
  name: string
  bundleId?: string
  icon?: string           // base64-encoded PNG
  pid?: number
}

export interface DiskInfo {
  total: number           // bytes
  used: number            // bytes
  available: number       // bytes
  usage: number           // 0-100
  mountPoint: string
}

export interface BrightnessInfo {
  brightness: number      // 0.0-1.0
  displayName?: string
}

export interface BluetoothDevice {
  name: string
  address: string
  connected: boolean
  deviceType?: string
  batteryLevel?: number   // 0-100
}

export interface BluetoothInfo {
  enabled: boolean
  devices: BluetoothDevice[]
}
