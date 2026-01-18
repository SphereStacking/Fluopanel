import type { DateInfo, Provider } from './types'

export interface DateProvider extends Provider<DateInfo> {
  getDate(format?: string): DateInfo
  startPolling(callback: (info: DateInfo) => void, interval?: number): () => void
}

export function createDateProvider(): DateProvider {
  const formatDate = (date: Date, format: string = 'HH:mm'): string => {
    const pad = (n: number) => n.toString().padStart(2, '0')

    const replacements: Record<string, string> = {
      'YYYY': date.getFullYear().toString(),
      'YY': date.getFullYear().toString().slice(-2),
      'MM': pad(date.getMonth() + 1),
      'M': (date.getMonth() + 1).toString(),
      'DD': pad(date.getDate()),
      'D': date.getDate().toString(),
      'HH': pad(date.getHours()),
      'H': date.getHours().toString(),
      'hh': pad(date.getHours() % 12 || 12),
      'h': (date.getHours() % 12 || 12).toString(),
      'mm': pad(date.getMinutes()),
      'm': date.getMinutes().toString(),
      'ss': pad(date.getSeconds()),
      's': date.getSeconds().toString(),
      'A': date.getHours() < 12 ? 'AM' : 'PM',
      'a': date.getHours() < 12 ? 'am' : 'pm',
      'ddd': ['Sun', 'Mon', 'Tue', 'Wed', 'Thu', 'Fri', 'Sat'][date.getDay()],
      'dddd': ['Sunday', 'Monday', 'Tuesday', 'Wednesday', 'Thursday', 'Friday', 'Saturday'][date.getDay()],
    }

    let result = format
    for (const [key, value] of Object.entries(replacements)) {
      result = result.replace(new RegExp(key, 'g'), value)
    }
    return result
  }

  let format = 'HH:mm'

  return {
    async get() {
      return this.getDate(format)
    },

    getDate(fmt?: string) {
      const now = new Date()
      const f = fmt ?? format
      return {
        timestamp: now.getTime(),
        formatted: formatDate(now, f)
      }
    },

    subscribe(callback) {
      return this.startPolling(callback)
    },

    startPolling(callback, interval = 1000) {
      const tick = () => {
        callback(this.getDate(format))
      }

      tick()
      const intervalId = setInterval(tick, interval)

      return () => {
        clearInterval(intervalId)
      }
    }
  }
}
