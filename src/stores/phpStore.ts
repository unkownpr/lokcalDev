import { create } from "zustand"
import { toast } from "sonner"
import type { PhpVersion, PhpIniDirective, PhpExtension } from "@/types/php"
import * as tauri from "@/lib/tauri"

interface PhpStore {
  versions: PhpVersion[]
  extensions: PhpExtension[]
  iniDirectives: PhpIniDirective[]
  loading: boolean
  installing: string | null
  error: string | null
  fetchVersions: () => Promise<void>
  installVersion: (version: string) => Promise<void>
  removeVersion: (version: string) => Promise<void>
  startFpm: (version: string) => Promise<void>
  stopFpm: (version: string) => Promise<void>
  restartFpm: (version: string) => Promise<void>
  fetchExtensions: (version: string) => Promise<void>
  toggleExtension: (version: string, extension: string, enable: boolean) => Promise<void>
  fetchIni: (version: string) => Promise<void>
  setIniDirective: (version: string, key: string, value: string) => Promise<void>
}

export const usePhpStore = create<PhpStore>((set, get) => ({
  versions: [],
  extensions: [],
  iniDirectives: [],
  loading: false,
  installing: null,
  error: null,

  fetchVersions: async () => {
    set({ loading: true, error: null })
    try {
      const versions = await tauri.phpListVersions()
      set({ versions, loading: false })
    } catch (err) {
      set({ error: String(err), loading: false })
    }
  },

  installVersion: async (version: string) => {
    set({ installing: version, error: null })
    try {
      await tauri.phpInstallVersion(version)
      await get().fetchVersions()
      set({ installing: null })
      toast.success(`PHP ${version} installed successfully`)
    } catch (err) {
      const msg = String(err)
      set({ error: msg, installing: null })
      toast.error(`Failed to install PHP ${version}`, { description: msg })
    }
  },

  removeVersion: async (version: string) => {
    try {
      await tauri.phpRemoveVersion(version)
      await get().fetchVersions()
      toast.success(`PHP ${version} removed`)
    } catch (err) {
      const msg = String(err)
      set({ error: msg })
      toast.error(`Failed to remove PHP ${version}`, { description: msg })
    }
  },

  startFpm: async (version: string) => {
    try {
      await tauri.phpStartFpm(version)
      await get().fetchVersions()
      toast.success(`PHP-FPM ${version} started`)
    } catch (err) {
      const msg = String(err)
      set({ error: msg })
      toast.error(`Failed to start PHP-FPM ${version}`, { description: msg })
    }
  },

  stopFpm: async (version: string) => {
    try {
      await tauri.phpStopFpm(version)
      await get().fetchVersions()
      toast.success(`PHP-FPM ${version} stopped`)
    } catch (err) {
      const msg = String(err)
      set({ error: msg })
      toast.error(`Failed to stop PHP-FPM ${version}`, { description: msg })
    }
  },

  restartFpm: async (version: string) => {
    try {
      await tauri.phpRestartFpm(version)
      await get().fetchVersions()
      toast.success(`PHP-FPM ${version} restarted`)
    } catch (err) {
      const msg = String(err)
      set({ error: msg })
      toast.error(`Failed to restart PHP-FPM ${version}`, { description: msg })
    }
  },

  fetchExtensions: async (version: string) => {
    try {
      const extensions = await tauri.phpListExtensions(version)
      set({ extensions })
    } catch (err) {
      set({ error: String(err) })
    }
  },

  toggleExtension: async (version: string, extension: string, enable: boolean) => {
    try {
      await tauri.phpToggleExtension(version, extension, enable)
      await get().fetchExtensions(version)
    } catch (err) {
      set({ error: String(err) })
    }
  },

  fetchIni: async (version: string) => {
    try {
      const iniDirectives = await tauri.phpGetIni(version)
      set({ iniDirectives })
    } catch (err) {
      set({ error: String(err) })
    }
  },

  setIniDirective: async (version: string, key: string, value: string) => {
    try {
      await tauri.phpSetIniDirective(version, key, value)
      await get().fetchIni(version)
    } catch (err) {
      set({ error: String(err) })
    }
  },
}))
