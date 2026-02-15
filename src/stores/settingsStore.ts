import { create } from "zustand"
import type { AppConfig } from "@/types/config"
import * as tauri from "@/lib/tauri"

interface SettingsStore {
  config: AppConfig | null
  loading: boolean
  error: string | null
  fetchSettings: () => Promise<void>
  saveSettings: (config: AppConfig) => Promise<void>
  resetSettings: () => Promise<void>
}

export const useSettingsStore = create<SettingsStore>((set) => ({
  config: null,
  loading: false,
  error: null,

  fetchSettings: async () => {
    set({ loading: true, error: null })
    try {
      const config = await tauri.settingsGet()
      set({ config, loading: false })
    } catch (err) {
      set({ error: String(err), loading: false })
    }
  },

  saveSettings: async (config: AppConfig) => {
    set({ loading: true, error: null })
    try {
      await tauri.settingsSave(config)
      set({ config, loading: false })
    } catch (err) {
      set({ error: String(err), loading: false })
    }
  },

  resetSettings: async () => {
    set({ loading: true, error: null })
    try {
      const config = await tauri.settingsReset()
      set({ config, loading: false })
    } catch (err) {
      set({ error: String(err), loading: false })
    }
  },
}))
