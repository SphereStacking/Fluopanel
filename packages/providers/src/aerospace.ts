import { invoke } from '@tauri-apps/api/core'
import { listen, type UnlistenFn } from '@tauri-apps/api/event'
import type { Workspace, AppIcon, Provider } from './types'

export interface FocusChangeEvent {
  focused: Workspace | null
  prev: Workspace | null
}

export interface AerospaceProvider extends Provider<Workspace[]> {
  getWorkspaces(): Promise<Workspace[]>
  getFocusedWorkspace(): Promise<Workspace | null>
  focusWorkspace(id: string): Promise<void>
  onWorkspaceChange(callback: (workspaces: Workspace[]) => void): () => void
  onFocusChange(callback: (event: FocusChangeEvent) => void): () => void
  getAppIcon(appName: string): Promise<AppIcon>
  getAppIcons(appNames: string[]): Promise<AppIcon[]>
}

export function createAerospaceProvider(): AerospaceProvider {
  let unlistenFn: UnlistenFn | null = null
  let subscribers: Set<(workspaces: Workspace[]) => void> = new Set()

  // Focus change listener
  let focusUnlistenFn: UnlistenFn | null = null
  let focusSubscribers: Set<(event: FocusChangeEvent) => void> = new Set()

  // Client-side icon cache to avoid repeated IPC calls
  const iconCache = new Map<string, string | null>()

  const setupListener = async () => {
    if (unlistenFn) return

    unlistenFn = await listen<Workspace[]>('aerospace-workspace-changed', (event) => {
      subscribers.forEach((callback) => callback(event.payload))
    })
  }

  const setupFocusListener = async () => {
    if (focusUnlistenFn) return

    focusUnlistenFn = await listen<FocusChangeEvent>('aerospace-focus-changed', (event) => {
      focusSubscribers.forEach((callback) => callback(event.payload))
    })
  }

  return {
    async get() {
      return this.getWorkspaces()
    },

    async getWorkspaces() {
      return invoke<Workspace[]>('aerospace_get_workspaces')
    },

    async getFocusedWorkspace() {
      return invoke<Workspace | null>('aerospace_get_focused_workspace')
    },

    async focusWorkspace(id: string) {
      return invoke('aerospace_focus_workspace', { id })
    },

    async getAppIcon(appName: string): Promise<AppIcon> {
      // Check client cache first
      if (iconCache.has(appName)) {
        return { app: appName, icon: iconCache.get(appName) ?? null }
      }

      const result = await invoke<AppIcon>('get_app_icon', { app_name: appName })
      iconCache.set(appName, result.icon)
      return result
    },

    async getAppIcons(appNames: string[]): Promise<AppIcon[]> {
      // Filter out already cached
      const uncached = appNames.filter((name) => !iconCache.has(name))

      if (uncached.length > 0) {
        const results = await invoke<AppIcon[]>('get_app_icons', { app_names: uncached })
        results.forEach((r) => iconCache.set(r.app, r.icon))
      }

      return appNames.map((name) => ({
        app: name,
        icon: iconCache.get(name) ?? null,
      }))
    },

    subscribe(callback) {
      return this.onWorkspaceChange(callback)
    },

    onWorkspaceChange(callback) {
      subscribers.add(callback)
      setupListener()

      return () => {
        subscribers.delete(callback)
        if (subscribers.size === 0 && unlistenFn) {
          unlistenFn()
          unlistenFn = null
        }
      }
    },

    onFocusChange(callback) {
      focusSubscribers.add(callback)
      setupFocusListener()

      return () => {
        focusSubscribers.delete(callback)
        if (focusSubscribers.size === 0 && focusUnlistenFn) {
          focusUnlistenFn()
          focusUnlistenFn = null
        }
      }
    },
  }
}
