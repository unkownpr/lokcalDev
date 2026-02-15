import { create } from "zustand"
import type { SystemInfo } from "@/types/config"
import * as tauri from "@/lib/tauri"

interface AppStore {
  initialized: boolean
  systemInfo: SystemInfo | null
  error: string | null
  initialize: () => Promise<void>
}

export const useAppStore = create<AppStore>((set) => ({
  initialized: false,
  systemInfo: null,
  error: null,

  initialize: async () => {
    try {
      await tauri.initializeApp()
      const systemInfo = await tauri.getSystemInfo()
      set({ initialized: true, systemInfo })
    } catch (err) {
      set({ error: String(err) })
    }
  },
}))
