import type { YouTubeMusicInfo, Provider } from './types'

const PEAR_API = 'http://localhost:26538'
const APP_ID = 'arcana'

export interface YouTubeMusicProvider extends Provider<YouTubeMusicInfo> {
  getInfo(): Promise<YouTubeMusicInfo>
  toggle(): Promise<void>
  next(): Promise<void>
  previous(): Promise<void>
  seek(seconds: number): Promise<void>
}

interface PearSongResponse {
  title?: string
  artist?: string
  album?: string
  isPaused?: boolean
  songDuration?: number
  elapsedSeconds?: number
  imageSrc?: string
}

interface AuthResponse {
  accessToken: string
}

export function createYouTubeMusicProvider(): YouTubeMusicProvider {
  let intervalId: ReturnType<typeof setInterval> | null = null
  const subscribers = new Set<(info: YouTubeMusicInfo) => void>()
  let lastInfo: YouTubeMusicInfo = { playing: false }
  let consecutiveFailures = 0
  let token: string | null = null
  const MAX_FAILURES = 3

  const authenticate = async (): Promise<string | null> => {
    try {
      const res = await fetch(`${PEAR_API}/auth/${APP_ID}`, { method: 'POST' })
      if (!res.ok) return null
      const data: AuthResponse = await res.json()
      return data.accessToken
    } catch {
      return null
    }
  }

  const getToken = async (): Promise<string | null> => {
    if (token) return token
    token = await authenticate()
    return token
  }

  const fetchInfo = async (): Promise<YouTubeMusicInfo | null> => {
    try {
      const accessToken = await getToken()
      if (!accessToken) throw new Error('No token')

      const controller = new AbortController()
      const timeoutId = setTimeout(() => controller.abort(), 500)

      const res = await fetch(`${PEAR_API}/api/v1/song-info`, {
        signal: controller.signal,
        headers: { Authorization: `Bearer ${accessToken}` },
      })
      clearTimeout(timeoutId)

      if (res.status === 401) {
        token = null
        throw new Error('Unauthorized')
      }
      if (!res.ok) throw new Error('Not available')

      consecutiveFailures = 0
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
      consecutiveFailures++
      return null
    }
  }

  const sendCommand = async (cmd: string): Promise<void> => {
    try {
      const accessToken = await getToken()
      if (!accessToken) return
      await fetch(`${PEAR_API}/api/v1/${cmd}`, {
        method: 'POST',
        headers: { Authorization: `Bearer ${accessToken}` },
      })
    } catch {
      // Pear Desktop not running - silently ignore
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
          // Pear Desktop が起動していない場合はポーリング停止
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
      await sendCommand('toggle-play')
    },

    async next() {
      await sendCommand('next')
    },

    async previous() {
      await sendCommand('previous')
    },

    async seek(seconds: number) {
      try {
        const accessToken = await getToken()
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
        // Pear Desktop not running - silently ignore
      }
    },
  }
}
