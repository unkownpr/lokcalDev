import { create } from "zustand"
import { listen, type UnlistenFn } from "@tauri-apps/api/event"
import type { LogFile, LogLine } from "@/types/ssl"
import * as tauri from "@/lib/tauri"

interface LogStore {
  files: LogFile[]
  lines: string[]
  selectedFile: string | null
  tailing: boolean
  error: string | null
  _unlisten: UnlistenFn | null
  fetchFiles: () => Promise<void>
  readFile: (path: string) => Promise<void>
  startTailing: (path: string) => Promise<void>
  stopTailing: () => Promise<void>
  clearFile: (path: string) => Promise<void>
  setSelectedFile: (path: string | null) => void
}

export const useLogStore = create<LogStore>((set, get) => ({
  files: [],
  lines: [],
  selectedFile: null,
  tailing: false,
  error: null,
  _unlisten: null,

  fetchFiles: async () => {
    try {
      const files = await tauri.logListFiles()
      set({ files })
    } catch (err) {
      set({ error: String(err) })
    }
  },

  readFile: async (path: string) => {
    try {
      const lines = await tauri.logReadFile(path)
      set({ lines, selectedFile: path })
    } catch (err) {
      set({ error: String(err) })
    }
  },

  startTailing: async (path: string) => {
    try {
      // Clean up existing listener first
      const existing = get()._unlisten
      if (existing) {
        existing()
        set({ _unlisten: null })
      }

      await tauri.logStartTailing(path)

      const unlisten = await listen<LogLine>("log-line", (event) => {
        set((state) => ({
          lines: [...state.lines, event.payload.line].slice(-1000),
        }))
      })

      set({ tailing: true, _unlisten: unlisten })
    } catch (err) {
      set({ error: String(err) })
    }
  },

  stopTailing: async () => {
    try {
      // Clean up listener
      const unlisten = get()._unlisten
      if (unlisten) {
        unlisten()
      }

      await tauri.logStopTailing()
      set({ tailing: false, _unlisten: null })
    } catch (err) {
      set({ error: String(err) })
    }
  },

  clearFile: async (path: string) => {
    try {
      await tauri.logClearFile(path)
      set({ lines: [] })
    } catch (err) {
      set({ error: String(err) })
    }
  },

  setSelectedFile: (path) => set({ selectedFile: path }),
}))
