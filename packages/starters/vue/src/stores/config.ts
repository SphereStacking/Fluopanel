import { ref, readonly } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import type { FluopanelConfig } from 'fluopanel-core'

const defaultConfig: FluopanelConfig = {
  version: 2,
  theme: {
    mode: 'system',
    accentColor: '#007AFF',
  },
  settings: {
    hotReload: true,
    devMode: false,
  },
  secrets: undefined,
}

const config = ref<FluopanelConfig>(defaultConfig)
const isLoading = ref(false)
const error = ref<string | null>(null)

export function useConfig() {
  const loadConfig = async () => {
    isLoading.value = true
    error.value = null
    try {
      const loaded = await invoke<FluopanelConfig>('get_config')
      config.value = { ...defaultConfig, ...loaded }
    } catch (e) {
      error.value = e instanceof Error ? e.message : String(e)
      console.error('Failed to load config:', e)
    } finally {
      isLoading.value = false
    }
  }

  const saveConfig = async (newConfig: Partial<FluopanelConfig>) => {
    const merged = { ...config.value, ...newConfig }
    try {
      await invoke('save_config', { config: merged })
      config.value = merged
    } catch (e) {
      error.value = e instanceof Error ? e.message : String(e)
      console.error('Failed to save config:', e)
      throw e
    }
  }

  const updateThemeMode = async (mode: 'light' | 'dark' | 'system') => {
    await saveConfig({
      theme: { ...config.value.theme, mode },
    })
  }

  const updateAccentColor = async (accentColor: string) => {
    await saveConfig({
      theme: { ...config.value.theme, accentColor },
    })
  }

  const setGitHubToken = async (token: string | undefined) => {
    await saveConfig({
      secrets: token ? { github: { token } } : undefined,
    })
  }

  return {
    config: readonly(config),
    isLoading: readonly(isLoading),
    error: readonly(error),
    loadConfig,
    saveConfig,
    updateThemeMode,
    updateAccentColor,
    setGitHubToken,
  }
}
