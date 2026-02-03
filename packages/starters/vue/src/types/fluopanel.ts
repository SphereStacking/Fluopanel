export type CSSValue = number | `${number}px` | `${number}vw` | `${number}vh` | `calc(${string})`

// CSS absoluteライクな位置指定
export interface PositionOptions {
  top?: CSSValue
  left?: CSSValue
  right?: CSSValue
  bottom?: CSSValue
  width?: CSSValue
  height?: CSSValue
  monitor?: string // モニター名 (例: 'LG HDR 4K', 'Built-in Retina Display')
}

// 初期化オプション（将来拡張可能）
export interface FluopanelInitOptions {
  position: PositionOptions
  // theme?: 'light' | 'dark' | 'system'
  // locale?: string
}

// MonitorInfo is exported from fluopanel-core
