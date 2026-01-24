import { ref, onMounted, onUnmounted } from 'vue'
import { createAerospaceProvider, type Workspace, type FocusChangeEvent, type AerospaceProvider } from '@arcana/providers'

export function useAerospaceProvider() {
  const workspaces = ref<Workspace[]>([])
  const appIcons = ref<Map<string, string | null>>(new Map())
  let provider: AerospaceProvider | null = null
  let unsubscribeFocus: (() => void) | null = null
  let unsubscribeWorkspace: (() => void) | null = null

  const fetchIconsForWorkspaces = async (ws: Workspace[]) => {
    if (!provider) return
    const appNames = new Set<string>()
    ws.forEach((w) => w.windows.forEach((win) => appNames.add(win.app)))
    const uncached = [...appNames].filter((app) => !appIcons.value.has(app))
    if (uncached.length > 0) {
      const icons = await provider.getAppIcons(uncached)
      // Create new Map to trigger Vue reactivity
      const newMap = new Map(appIcons.value)
      icons.forEach((icon) => {
        newMap.set(icon.app, icon.icon)
      })
      appIcons.value = newMap
    }
  }

  const refresh = async () => {
    if (!provider) return
    try {
      const result = await provider.getWorkspaces()
      workspaces.value = result
      fetchIconsForWorkspaces(result)
    } catch (error) {
      console.error('Failed to refresh workspaces:', error)
    }
  }

  onMounted(async () => {
    provider = createAerospaceProvider()
    await refresh()

    unsubscribeFocus = provider.onFocusChange(async ({ focused, prev }: FocusChangeEvent) => {
      workspaces.value = workspaces.value.map((ws) => {
        if (focused && ws.id === focused.id) return focused
        if (prev && ws.id === prev.id) return { ...prev, focused: false }
        return { ...ws, focused: false }
      })

      const newApps = new Set<string>()
      if (focused) focused.windows.forEach((w) => newApps.add(w.app))
      if (prev) prev.windows.forEach((w) => newApps.add(w.app))

      const uncached = [...newApps].filter((app) => !appIcons.value.has(app))
      if (uncached.length > 0 && provider) {
        const icons = await provider.getAppIcons(uncached)
        // Create new Map to trigger Vue reactivity
        const newMap = new Map(appIcons.value)
        icons.forEach((icon) => {
          newMap.set(icon.app, icon.icon)
        })
        appIcons.value = newMap
      }
    })

    unsubscribeWorkspace = provider.onWorkspaceChange(async (ws) => {
      workspaces.value = ws
      await fetchIconsForWorkspaces(ws)
    })
  })

  onUnmounted(() => {
    unsubscribeFocus?.()
    unsubscribeWorkspace?.()
  })

  const focusWorkspace = async (id: string) => {
    await provider?.focusWorkspace(id)
    await refresh()
  }

  const getAppIcon = (appName: string): string | null => {
    return appIcons.value.get(appName) ?? null
  }

  return { workspaces, appIcons, focusWorkspace, getAppIcon, refresh }
}
