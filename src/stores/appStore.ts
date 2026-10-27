import { create } from "zustand"
import { toast } from "sonner"
import type { SystemInfo } from "@/types/config"
import * as tauri from "@/lib/tauri"
import { useSettingsStore } from "@/stores/settingsStore"

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

/** Start MariaDB and Nginx if installed and stopped; Nginx starts the PHP-FPM versions its sites need. */
async function autoStartServices() {
  try {
    const services = await tauri.getAllServices()
    for (const id of ["mariadb", "nginx"]) {
      const svc = services.find((s) => s.id === id)
      if (!svc || !svc.installed || svc.status === "running") continue
      if (id === "mariadb" && !svc.initialized) continue
      await tauri.startService(id).catch((err) => {
        toast.error(`Auto-start failed for ${svc.name}`, { description: String(err) })
      })
    }
  } catch (err) {
    console.warn("Auto-start services failed:", err)
  }
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

      await useSettingsStore.getState().fetchSettings()
      const settings = useSettingsStore.getState().config

      // Ensure dnsmasq is running for local domain resolution (no password needed)
      tauri.dnsEnsureDnsmasqRunning(settings?.tld ?? "test").catch(() => {})

      if (settings?.autoStartServices) {
        autoStartServices()
      }

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
        toast.success(`Update available: v${update.version}`)
      } else {
        set({
          updateAvailable: false,
          updateVersion: null,
          updateNotes: null,
          updateChecking: false,
        })
        toast.success("You're on the latest version")
      }
    } catch {
      set({ updateChecking: false })
      toast.error("Could not check for updates")
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
