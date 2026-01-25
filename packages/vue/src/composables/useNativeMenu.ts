import { toValue, watch, isRef, type MaybeRef } from 'vue'
import { Menu, MenuItem, Submenu, PredefinedMenuItem, CheckMenuItem } from '@tauri-apps/api/menu'
import { getCurrentWindow } from '@tauri-apps/api/window'

// --- Type Definitions ---

export interface NativeMenuItemBase {
  id: string
  label: string
  accelerator?: string
  enabled?: boolean
}

export interface NativeMenuAction extends NativeMenuItemBase {
  type?: 'item'
}

export interface NativeMenuSeparator {
  type: 'separator'
}

export interface NativeMenuSubmenu extends NativeMenuItemBase {
  type: 'submenu'
  items: NativeMenuItemDef[]
}

export interface NativeMenuCheck extends NativeMenuItemBase {
  type: 'check'
  checked: boolean
}

export type NativeMenuItemDef =
  | NativeMenuAction
  | NativeMenuSeparator
  | NativeMenuSubmenu
  | NativeMenuCheck

export interface UseNativeMenuOptions {
  /** Menu items - can be reactive (ref) or plain array */
  items: MaybeRef<NativeMenuItemDef[]>
  /** Callback when a menu item is selected */
  onSelect?: (id: string, item: NativeMenuItemDef) => void | Promise<void>
}

export interface UseNativeMenuReturn {
  /** Show the menu at cursor position */
  popup: () => Promise<void>
}

// --- Implementation ---

export function useNativeMenu(options: UseNativeMenuOptions): UseNativeMenuReturn {
  let cachedMenu: Menu | null = null
  let menuDirty = true

  // Watch for reactive items changes
  if (isRef(options.items)) {
    watch(
      options.items,
      () => {
        menuDirty = true
      },
      { deep: true }
    )
  }

  async function buildMenuItem(
    def: NativeMenuItemDef,
    onSelect?: (id: string, item: NativeMenuItemDef) => void | Promise<void>
  ): Promise<MenuItem | Submenu | PredefinedMenuItem | CheckMenuItem> {
    if (def.type === 'separator') {
      return await PredefinedMenuItem.new({ item: 'Separator' })
    }

    if (def.type === 'submenu') {
      const subItems = await Promise.all(
        def.items.map((item) => buildMenuItem(item, onSelect))
      )
      return await Submenu.new({
        text: def.label,
        enabled: def.enabled ?? true,
        items: subItems,
      })
    }

    if (def.type === 'check') {
      return await CheckMenuItem.new({
        id: def.id,
        text: def.label,
        enabled: def.enabled ?? true,
        accelerator: def.accelerator,
        checked: def.checked,
        action: () => onSelect?.(def.id, def),
      })
    }

    return await MenuItem.new({
      id: def.id,
      text: def.label,
      enabled: def.enabled ?? true,
      accelerator: def.accelerator,
      action: () => onSelect?.(def.id, def),
    })
  }

  async function ensureMenu(): Promise<Menu> {
    if (cachedMenu && !menuDirty) {
      return cachedMenu
    }

    const items = toValue(options.items)
    const menuItems = await Promise.all(
      items.map((item) => buildMenuItem(item, options.onSelect))
    )

    cachedMenu = await Menu.new({ items: menuItems })
    menuDirty = false
    return cachedMenu
  }

  async function popup(): Promise<void> {
    const menu = await ensureMenu()
    const window = getCurrentWindow()
    await menu.popup(undefined, window)
  }

  return { popup }
}
