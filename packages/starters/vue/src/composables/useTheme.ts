import { ref, onMounted, onUnmounted, computed } from 'vue'

export type ThemeMode = 'light' | 'dark' | 'system'

export function useTheme(initialMode: ThemeMode = 'system') {
  const mode = ref<ThemeMode>(initialMode)
  const systemTheme = ref<'light' | 'dark'>('dark')

  const currentTheme = computed(() => {
    if (mode.value === 'system') {
      return systemTheme.value
    }
    return mode.value
  })

  const updateSystemTheme = () => {
    const isDark = window.matchMedia('(prefers-color-scheme: dark)').matches
    systemTheme.value = isDark ? 'dark' : 'light'
  }

  let mediaQuery: MediaQueryList | null = null

  onMounted(() => {
    updateSystemTheme()
    mediaQuery = window.matchMedia('(prefers-color-scheme: dark)')
    mediaQuery.addEventListener('change', updateSystemTheme)
  })

  onUnmounted(() => {
    if (mediaQuery) {
      mediaQuery.removeEventListener('change', updateSystemTheme)
    }
  })

  const setMode = (newMode: ThemeMode) => {
    mode.value = newMode
  }

  return {
    mode,
    currentTheme,
    setMode,
  }
}
