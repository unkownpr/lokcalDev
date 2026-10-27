import { create } from "zustand"
import { toast } from "sonner"
import type { AppConfig } from "@/types/config"
import * as tauri from "@/lib/tauri"

interface SettingsStore {
  config: AppConfig | null
  loading: boolean
  error: string | null
  fetchSettings: () => Promise<void>
  /** Resolves to false (and shows a toast) when the backend rejects the config. */
  saveSettings: (config: AppConfig) => Promise<boolean>
  resetSettings: () => Promise<void>
}

export const useSettingsStore = create<SettingsStore>((set, get) => ({
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
    const previousTld = get().config?.tld
    try {
      await tauri.settingsSave(config)
      set({ config, loading: false })
    } catch (err) {
      set({ error: String(err), loading: false })
      toast.error("Failed to save settings", { description: String(err) })
      return false
    }
    if (previousTld !== config.tld) {
      // Add the dnsmasq wildcard for the new TLD (resolver file still needs SSL > DNS setup)
      tauri.dnsEnsureDnsmasqRunning(config.tld).catch(() => {})
    }
    return true
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

/** Configured TLD for local domains; loads settings on first use. */
export async function getTld(): Promise<string> {
  if (!useSettingsStore.getState().config) {
    await useSettingsStore.getState().fetchSettings()
  }
  return useSettingsStore.getState().config?.tld ?? "test"
}
