import { ref, readonly } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import type { AppConfig } from '@arcana/core'

const defaultConfig: AppConfig = {
  bar: {
    position: 'top',
    height: 32,
    opacity: 0.9,
  },
  widgets: {
    workspaces: { enabled: true },
    clock: { enabled: true, format: 'HH:mm' },
    battery: { enabled: true },
    cpu: { enabled: true },
    memory: { enabled: true },
    network: { enabled: true },
  },
  theme: {
    mode: 'system',
  },
  github: {
    token: undefined,
  },
}

const config = ref<AppConfig>(defaultConfig)
const isLoading = ref(false)
const error = ref<string | null>(null)

export function useConfig() {
  const loadConfig = async () => {
    isLoading.value = true
    error.value = null
    try {
      const loaded = await invoke<AppConfig>('get_config')
      config.value = { ...defaultConfig, ...loaded }
    } catch (e) {
      error.value = e instanceof Error ? e.message : String(e)
      console.error('Failed to load config:', e)
    } finally {
      isLoading.value = false
    }
  }

  const saveConfig = async (newConfig: Partial<AppConfig>) => {
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

  const updateBarPosition = async (position: 'top' | 'bottom') => {
    await saveConfig({
      bar: { ...config.value.bar, position },
    })
  }

  const updateBarHeight = async (height: number) => {
    await saveConfig({
      bar: { ...config.value.bar, height },
    })
  }

  const toggleWidget = async (
    widget: keyof AppConfig['widgets'],
    enabled: boolean
  ) => {
    await saveConfig({
      widgets: {
        ...config.value.widgets,
        [widget]: { ...config.value.widgets[widget], enabled },
      },
    })
  }

  return {
    config: readonly(config),
    isLoading: readonly(isLoading),
    error: readonly(error),
    loadConfig,
    saveConfig,
    updateBarPosition,
    updateBarHeight,
    toggleWidget,
  }
}
