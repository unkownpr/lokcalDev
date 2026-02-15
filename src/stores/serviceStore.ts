import { create } from "zustand"
import { toast } from "sonner"
import type { ServiceInfo } from "@/types/service"
import * as tauri from "@/lib/tauri"

interface ServiceStore {
  services: ServiceInfo[]
  loading: boolean
  error: string | null
  fetchServices: () => Promise<void>
  installService: (id: string) => Promise<void>
  initializeService: (id: string) => Promise<void>
  startService: (id: string) => Promise<void>
  stopService: (id: string) => Promise<void>
  restartService: (id: string) => Promise<void>
}

export const useServiceStore = create<ServiceStore>((set, get) => ({
  services: [],
  loading: false,
  error: null,

  fetchServices: async () => {
    set({ loading: true, error: null })
    try {
      const services = await tauri.getAllServices()
      set({ services, loading: false })
    } catch (err) {
      set({ error: String(err), loading: false })
    }
  },

  installService: async (id: string) => {
    try {
      if (id === "nginx") {
        await tauri.nginxInstall()
        toast.success("Nginx installed successfully")
      } else if (id === "mariadb") {
        await tauri.mariadbInstall()
        toast.success("MariaDB installed successfully")
      } else if (id === "phpmyadmin") {
        await tauri.phpmyadminInstall()
        toast.success("phpMyAdmin installed successfully")
      }
      await get().fetchServices()
    } catch (err) {
      const msg = String(err)
      set({ error: msg })
      toast.error("Failed to install service", { description: msg })
    }
  },

  initializeService: async (id: string) => {
    try {
      if (id === "mariadb") {
        await tauri.mariadbInitialize()
        toast.success("MariaDB initialized")
      }
      await get().fetchServices()
    } catch (err) {
      const msg = String(err)
      set({ error: msg })
      toast.error("Failed to initialize service", { description: msg })
    }
  },

  startService: async (id: string) => {
    try {
      await tauri.startService(id)
      await get().fetchServices()
      toast.success("Service started")
    } catch (err) {
      const msg = String(err)
      set({ error: msg })
      toast.error("Failed to start service", { description: msg })
    }
  },

  stopService: async (id: string) => {
    try {
      await tauri.stopService(id)
      await get().fetchServices()
      toast.success("Service stopped")
    } catch (err) {
      const msg = String(err)
      set({ error: msg })
      toast.error("Failed to stop service", { description: msg })
    }
  },

  restartService: async (id: string) => {
    try {
      await tauri.restartService(id)
      await get().fetchServices()
      toast.success("Service restarted")
    } catch (err) {
      const msg = String(err)
      set({ error: msg })
      toast.error("Failed to restart service", { description: msg })
    }
  },
}))
