import { create } from "zustand"
import { toast } from "sonner"
import type { MariaDbInfo, DatabaseEntry } from "@/types/database"
import * as tauri from "@/lib/tauri"

interface DatabaseStore {
  info: MariaDbInfo | null
  databases: DatabaseEntry[]
  loading: boolean
  error: string | null
  fetchInfo: () => Promise<void>
  install: () => Promise<void>
  initialize: () => Promise<void>
  start: () => Promise<void>
  stop: () => Promise<void>
  restart: () => Promise<void>
  fetchDatabases: () => Promise<void>
  createDatabase: (name: string) => Promise<void>
  dropDatabase: (name: string) => Promise<void>
}

export const useDatabaseStore = create<DatabaseStore>((set, get) => ({
  info: null,
  databases: [],
  loading: false,
  error: null,

  fetchInfo: async () => {
    try {
      const info = await tauri.mariadbGetInfo()
      set({ info })
    } catch (err) {
      set({ error: String(err) })
    }
  },

  install: async () => {
    set({ loading: true, error: null })
    try {
      await tauri.mariadbInstall()
      await get().fetchInfo()
      set({ loading: false })
      toast.success("MariaDB installed successfully")
    } catch (err) {
      const msg = String(err)
      set({ error: msg, loading: false })
      toast.error("Failed to install MariaDB", { description: msg })
    }
  },

  initialize: async () => {
    set({ loading: true, error: null })
    try {
      await tauri.mariadbInitialize()
      await get().fetchInfo()
      set({ loading: false })
      toast.success("MariaDB database initialized")
    } catch (err) {
      const msg = String(err)
      set({ error: msg, loading: false })
      toast.error("Failed to initialize MariaDB", { description: msg })
    }
  },

  start: async () => {
    try {
      await tauri.mariadbStart()
      await get().fetchInfo()
      toast.success("MariaDB started")
    } catch (err) {
      const msg = String(err)
      set({ error: msg })
      toast.error("Failed to start MariaDB", { description: msg })
    }
  },

  stop: async () => {
    try {
      await tauri.mariadbStop()
      await get().fetchInfo()
      toast.success("MariaDB stopped")
    } catch (err) {
      const msg = String(err)
      set({ error: msg })
      toast.error("Failed to stop MariaDB", { description: msg })
    }
  },

  restart: async () => {
    try {
      await tauri.mariadbRestart()
      await get().fetchInfo()
      toast.success("MariaDB restarted")
    } catch (err) {
      const msg = String(err)
      set({ error: msg })
      toast.error("Failed to restart MariaDB", { description: msg })
    }
  },

  fetchDatabases: async () => {
    try {
      const databases = await tauri.databaseList()
      set({ databases })
    } catch (err) {
      set({ error: String(err) })
    }
  },

  createDatabase: async (name: string) => {
    try {
      await tauri.databaseCreate(name)
      await get().fetchDatabases()
      toast.success(`Database "${name}" created`)
    } catch (err) {
      const msg = String(err)
      set({ error: msg })
      toast.error("Failed to create database", { description: msg })
    }
  },

  dropDatabase: async (name: string) => {
    try {
      await tauri.databaseDrop(name)
      await get().fetchDatabases()
      toast.success(`Database "${name}" dropped`)
    } catch (err) {
      const msg = String(err)
      set({ error: msg })
      toast.error("Failed to drop database", { description: msg })
    }
  },
}))
