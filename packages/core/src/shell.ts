import { invoke } from '@tauri-apps/api/core'

export async function executeShell(command: string): Promise<string> {
  return invoke<string>('execute_shell', { command })
}
