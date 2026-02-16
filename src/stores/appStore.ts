import { create } from "zustand"
import type { SystemInfo } from "@/types/config"
import * as tauri from "@/lib/tauri"

interface AppStore {
  initialized: boolean
  systemInfo: SystemInfo | null
  error: string | null
  updateAvailable: boolean
  updateVersion: string | null
  updateNotes: string | null
  updateChecking: boolean
  updateInstalling: boolean
  updateProgress: { downloaded: number; total: number | undefined } | null
  initialize: () => Promise<void>
  checkForUpdate: () => Promise<void>
  installUpdate: () => Promise<void>
}

export const useAppStore = create<AppStore>((set) => ({
  initialized: false,
  systemInfo: null,
  error: null,
  updateAvailable: false,
  updateVersion: null,
  updateNotes: null,
  updateChecking: false,
  updateInstalling: false,
  updateProgress: null,

  initialize: async () => {
    try {
      await tauri.initializeApp()
      const systemInfo = await tauri.getSystemInfo()
      set({ initialized: true, systemInfo })

      // Check for updates in background (non-blocking)
      // Pre-check endpoint to avoid Rust-side ERROR logs when no release exists
      fetch("https://github.com/unkownpr/lokcalDev/releases/latest/download/latest.json", { method: "HEAD" })
        .then((res) => {
          if (!res.ok) return
          return tauri.checkForUpdate()
        })
        .then((update) => {
          if (update) {
            set({
              updateAvailable: true,
              updateVersion: update.version,
              updateNotes: update.body,
            })
          }
        })
        .catch(() => {})
    } catch (err) {
      set({ error: String(err) })
    }
  },

  checkForUpdate: async () => {
    set({ updateChecking: true })
    try {
      const update = await tauri.checkForUpdate()
      if (update) {
        set({
          updateAvailable: true,
          updateVersion: update.version,
          updateNotes: update.body,
          updateChecking: false,
        })
      } else {
        set({
          updateAvailable: false,
          updateVersion: null,
          updateNotes: null,
          updateChecking: false,
        })
      }
    } catch {
      set({ updateChecking: false })
    }
  },

  installUpdate: async () => {
    set({ updateInstalling: true, updateProgress: null })
    try {
      await tauri.downloadAndInstallUpdate((downloaded, total) => {
        set({ updateProgress: { downloaded, total } })
      })
    } catch {
      set({ updateInstalling: false, updateProgress: null })
    }
  },
}))
