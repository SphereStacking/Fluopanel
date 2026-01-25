import type { YouTubeMusicInfo, Provider } from './types'

// Tauri global API type
declare const window: Window & {
  __TAURI__?: {
    core: {
      invoke: (cmd: string, args?: Record<string, unknown>) => Promise<unknown>
    }
  }
}

// Pear Desktop API (port 26538)
const PEAR_API = 'http://localhost:26538'
const PEAR_APP_ID = 'arcana'

// YTM Desktop API (port 9863)
const YTM_API = 'http://localhost:9863'

type ApiType = 'pear' | 'ytm' | null

export interface YouTubeMusicProvider extends Provider<YouTubeMusicInfo> {
  getInfo(): Promise<YouTubeMusicInfo>
  toggle(): Promise<void>
  next(): Promise<void>
  previous(): Promise<void>
  seek(seconds: number): Promise<void>
  launch(): Promise<void>
}

// Pear Desktop response format
interface PearSongResponse {
  title?: string
  artist?: string
  album?: string
  isPaused?: boolean
  songDuration?: number
  elapsedSeconds?: number
  imageSrc?: string
}

interface PearAuthResponse {
  accessToken: string
}

// YTM Desktop response format
interface YtmQueryResponse {
  player?: {
    trackState?: number // 0: paused, 1: playing, 2: buffering
    videoProgress?: number
    volume?: number
  }
  track?: {
    author?: string
    title?: string
    album?: string
    cover?: string
    duration?: number
    durationHuman?: string
  }
}

export function createYouTubeMusicProvider(): YouTubeMusicProvider {
  let intervalId: ReturnType<typeof setInterval> | null = null
  const subscribers = new Set<(info: YouTubeMusicInfo) => void>()
  let lastInfo: YouTubeMusicInfo = { playing: false }
  let consecutiveFailures = 0
  let pearToken: string | null = null
  let detectedApi: ApiType = null
  const MAX_FAILURES = 3

  // Pear Desktop authentication
  const authenticatePear = async (): Promise<string | null> => {
    try {
      const res = await fetch(`${PEAR_API}/auth/${PEAR_APP_ID}`, { method: 'POST' })
      if (!res.ok) return null
      const data: PearAuthResponse = await res.json()
      return data.accessToken
    } catch {
      return null
    }
  }

  const getPearToken = async (): Promise<string | null> => {
    if (pearToken) return pearToken
    pearToken = await authenticatePear()
    return pearToken
  }

  // Detect which API is available
  const detectApi = async (): Promise<ApiType> => {
    if (detectedApi) return detectedApi

    // Try YTM Desktop first (more common)
    try {
      const controller = new AbortController()
      const timeoutId = setTimeout(() => controller.abort(), 500)
      const res = await fetch(`${YTM_API}/query`, {
        signal: controller.signal,
      })
      clearTimeout(timeoutId)
      if (res.ok) {
        detectedApi = 'ytm'
        return 'ytm'
      }
    } catch {
      // YTM not available
    }

    // Try Pear Desktop
    try {
      const token = await getPearToken()
      if (token) {
        detectedApi = 'pear'
        return 'pear'
      }
    } catch {
      // Pear not available
    }

    return null
  }

  // Fetch info from Pear Desktop
  const fetchPearInfo = async (): Promise<YouTubeMusicInfo | null> => {
    try {
      const accessToken = await getPearToken()
      if (!accessToken) throw new Error('No token')

      const controller = new AbortController()
      const timeoutId = setTimeout(() => controller.abort(), 500)

      const res = await fetch(`${PEAR_API}/api/v1/song-info`, {
        signal: controller.signal,
        headers: { Authorization: `Bearer ${accessToken}` },
      })
      clearTimeout(timeoutId)

      if (res.status === 401) {
        pearToken = null
        throw new Error('Unauthorized')
      }
      if (!res.ok) throw new Error('Not available')

      const data: PearSongResponse = await res.json()
      return {
        playing: !data.isPaused,
        title: data.title,
        artist: data.artist,
        album: data.album,
        duration: data.songDuration,
        position: data.elapsedSeconds,
        artworkUrl: data.imageSrc,
      }
    } catch {
      return null
    }
  }

  // Fetch info from YTM Desktop
  const fetchYtmInfo = async (): Promise<YouTubeMusicInfo | null> => {
    try {
      const controller = new AbortController()
      const timeoutId = setTimeout(() => controller.abort(), 500)

      const res = await fetch(`${YTM_API}/query`, {
        signal: controller.signal,
      })
      clearTimeout(timeoutId)

      if (!res.ok) throw new Error('Not available')

      const data: YtmQueryResponse = await res.json()
      if (!data.track?.title) return { playing: false }

      return {
        playing: data.player?.trackState === 1,
        title: data.track.title,
        artist: data.track.author,
        album: data.track.album,
        duration: data.track.duration,
        position: data.player?.videoProgress,
        artworkUrl: data.track.cover,
      }
    } catch {
      return null
    }
  }

  const fetchInfo = async (): Promise<YouTubeMusicInfo | null> => {
    const api = await detectApi()

    let info: YouTubeMusicInfo | null = null
    if (api === 'ytm') {
      info = await fetchYtmInfo()
    } else if (api === 'pear') {
      info = await fetchPearInfo()
    }

    if (info) {
      consecutiveFailures = 0
    } else {
      consecutiveFailures++
      // Reset detected API on failure to allow re-detection
      if (consecutiveFailures >= MAX_FAILURES) {
        detectedApi = null
      }
    }

    return info
  }

  // Send command to Pear Desktop
  const sendPearCommand = async (cmd: string): Promise<void> => {
    try {
      const accessToken = await getPearToken()
      if (!accessToken) return
      await fetch(`${PEAR_API}/api/v1/${cmd}`, {
        method: 'POST',
        headers: { Authorization: `Bearer ${accessToken}` },
      })
    } catch {
      // Pear Desktop not running - silently ignore
    }
  }

  // Send command to YTM Desktop
  const sendYtmCommand = async (cmd: string): Promise<void> => {
    try {
      await fetch(`${YTM_API}/query`, {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ command: cmd }),
      })
    } catch {
      // YTM Desktop not running - silently ignore
    }
  }

  const sendCommand = async (pearCmd: string, ytmCmd: string): Promise<void> => {
    const api = await detectApi()
    if (api === 'ytm') {
      await sendYtmCommand(ytmCmd)
    } else if (api === 'pear') {
      await sendPearCommand(pearCmd)
    }
  }

  const hasChanged = (a: YouTubeMusicInfo, b: YouTubeMusicInfo): boolean => {
    return (
      a.playing !== b.playing ||
      a.title !== b.title ||
      a.artist !== b.artist ||
      a.album !== b.album
    )
  }

  return {
    async get() {
      return this.getInfo()
    },

    async getInfo() {
      const info = await fetchInfo()
      if (info) lastInfo = info
      return lastInfo
    },

    subscribe(callback) {
      subscribers.add(callback)
      consecutiveFailures = 0

      if (!intervalId) {
        intervalId = setInterval(async () => {
          // API が起動していない場合はポーリング停止
          if (consecutiveFailures >= MAX_FAILURES) {
            if (intervalId) {
              clearInterval(intervalId)
              intervalId = null
            }
            return
          }

          const info = await fetchInfo()
          if (info && hasChanged(info, lastInfo)) {
            lastInfo = info
            subscribers.forEach((cb) => cb(info))
          }
        }, 3000)
      }

      return () => {
        subscribers.delete(callback)
        if (subscribers.size === 0 && intervalId) {
          clearInterval(intervalId)
          intervalId = null
          consecutiveFailures = 0
        }
      }
    },

    async toggle() {
      await sendCommand('toggle-play', 'track-pause')
    },

    async next() {
      await sendCommand('next', 'track-next')
    },

    async previous() {
      await sendCommand('previous', 'track-previous')
    },

    async seek(seconds: number) {
      const api = await detectApi()
      if (api === 'ytm') {
        try {
          await fetch(`${YTM_API}/query`, {
            method: 'POST',
            headers: { 'Content-Type': 'application/json' },
            body: JSON.stringify({ command: 'player-seek-to', value: Math.floor(seconds) }),
          })
        } catch {
          // YTM Desktop not running
        }
      } else if (api === 'pear') {
        try {
          const accessToken = await getPearToken()
          if (!accessToken) return
          await fetch(`${PEAR_API}/api/v1/seek-to`, {
            method: 'POST',
            headers: {
              Authorization: `Bearer ${accessToken}`,
              'Content-Type': 'application/json',
            },
            body: JSON.stringify({ seconds: Math.floor(seconds) }),
          })
        } catch {
          // Pear Desktop not running
        }
      }
    },

    async launch() {
      if (!window.__TAURI__) {
        console.error('[YouTubeMusic] Tauri API not available')
        return
      }
      try {
        // Try to open YouTube Music app using macOS open command
        await window.__TAURI__.core.invoke('execute_shell', { command: 'open -a "YouTube Music"' })
      } catch (error) {
        console.error('[YouTubeMusic] Failed to launch app:', error)
        // Fallback: try opening the web version
        try {
          await window.__TAURI__.core.invoke('execute_shell', { command: 'open "https://music.youtube.com"' })
        } catch (e2) {
          console.error('[YouTubeMusic] Fallback also failed:', e2)
        }
      }
    },
  }
}
