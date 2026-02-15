import { create } from "zustand"
import { toast } from "sonner"
import type { Site, NginxInfo } from "@/types/nginx"
import * as tauri from "@/lib/tauri"

interface SiteStore {
  sites: Site[]
  nginxInfo: NginxInfo | null
  loading: boolean
  error: string | null
  fetchSites: () => Promise<void>
  fetchNginxInfo: () => Promise<void>
  createSite: (name: string, domain: string, documentRoot: string, phpVersion: string, ssl: boolean) => Promise<void>
  updateSite: (id: string, data: Partial<Site>) => Promise<void>
  deleteSite: (id: string) => Promise<void>
  installNginx: () => Promise<void>
  startNginx: () => Promise<void>
  stopNginx: () => Promise<void>
  restartNginx: () => Promise<void>
  reloadNginx: () => Promise<void>
}

export const useSiteStore = create<SiteStore>((set, get) => ({
  sites: [],
  nginxInfo: null,
  loading: false,
  error: null,

  fetchSites: async () => {
    set({ loading: true, error: null })
    try {
      const sites = await tauri.siteList()
      set({ sites, loading: false })
    } catch (err) {
      set({ error: String(err), loading: false })
    }
  },

  fetchNginxInfo: async () => {
    try {
      const nginxInfo = await tauri.nginxGetInfo()
      set({ nginxInfo })
    } catch (err) {
      set({ error: String(err) })
    }
  },

  createSite: async (name, domain, documentRoot, phpVersion, ssl) => {
    try {
      await tauri.siteCreate(name, domain, documentRoot, phpVersion, ssl)
    } catch (err) {
      const msg = String(err)
      set({ error: msg })
      toast.error("Failed to create site", { description: msg })
      return
    }

    let hasWarnings = false

    // Add DNS entry
    try {
      await tauri.dnsAddEntry(domain, "127.0.0.1")
    } catch (err) {
      hasWarnings = true
      toast.error("Site created but DNS entry failed", { description: String(err) })
    }

    // Generate SSL certificate if enabled
    if (ssl) {
      try {
        await tauri.sslGenerateCertificate(domain)
      } catch (err) {
        hasWarnings = true
        toast.error("SSL certificate generation failed", { description: String(err) })
      }
    }

    // Reload nginx to pick up new config
    try {
      await tauri.nginxReload()
    } catch (err) {
      hasWarnings = true
      toast.error("Nginx reload failed", { description: String(err) })
    }

    await get().fetchSites()
    if (!hasWarnings) {
      toast.success(`Site "${name}" created`, { description: `${ssl ? "https" : "http"}://${domain}` })
    }
  },

  updateSite: async (id, data) => {
    // Get old site to detect domain change
    const oldSite = get().sites.find((s) => s.id === id)

    try {
      await tauri.siteUpdate(id, data.name, data.domain, data.documentRoot, data.phpVersion, data.ssl, data.active)
    } catch (err) {
      const msg = String(err)
      set({ error: msg })
      toast.error("Failed to update site", { description: msg })
      return
    }

    // Handle domain change: remove old DNS, add new DNS
    if (data.domain && oldSite && data.domain !== oldSite.domain) {
      try {
        await tauri.dnsRemoveEntry(oldSite.domain)
      } catch {}
      try {
        await tauri.dnsAddEntry(data.domain, "127.0.0.1")
      } catch (err) {
        toast.error("DNS update failed", { description: String(err) })
      }
    }

    // Handle SSL change - only regenerate cert when SSL was toggled on or domain changed
    const newSsl = data.ssl ?? oldSite?.ssl
    const newDomain = data.domain ?? oldSite?.domain
    const sslToggled = data.ssl !== undefined && data.ssl !== oldSite?.ssl
    const domainChanged = data.domain !== undefined && data.domain !== oldSite?.domain
    if (newSsl && newDomain && (sslToggled || domainChanged)) {
      try {
        await tauri.sslGenerateCertificate(newDomain)
      } catch (err) {
        toast.error("SSL certificate generation failed", { description: String(err) })
      }
    }

    // Reload nginx
    try {
      await tauri.nginxReload()
    } catch {}

    await get().fetchSites()
    toast.success("Site updated")
  },

  deleteSite: async (id) => {
    const site = get().sites.find((s) => s.id === id)

    try {
      await tauri.siteDelete(id)
    } catch (err) {
      const msg = String(err)
      set({ error: msg })
      toast.error("Failed to delete site", { description: msg })
      return
    }

    // Remove DNS entry
    if (site) {
      try {
        await tauri.dnsRemoveEntry(site.domain)
      } catch {}
    }

    // Reload nginx
    try {
      await tauri.nginxReload()
    } catch {}

    await get().fetchSites()
    toast.success("Site deleted")
  },

  installNginx: async () => {
    try {
      await tauri.nginxInstall()
      await get().fetchNginxInfo()
      toast.success("Nginx installed successfully")
    } catch (err) {
      const msg = String(err)
      set({ error: msg })
      toast.error("Failed to install Nginx", { description: msg })
    }
  },

  startNginx: async () => {
    try {
      await tauri.nginxStart()
      await get().fetchNginxInfo()
      toast.success("Nginx started")
    } catch (err) {
      const msg = String(err)
      set({ error: msg })
      toast.error("Failed to start Nginx", { description: msg })
    }
  },

  stopNginx: async () => {
    try {
      await tauri.nginxStop()
      await get().fetchNginxInfo()
      toast.success("Nginx stopped")
    } catch (err) {
      const msg = String(err)
      set({ error: msg })
      toast.error("Failed to stop Nginx", { description: msg })
    }
  },

  restartNginx: async () => {
    try {
      await tauri.nginxRestart()
      await get().fetchNginxInfo()
      toast.success("Nginx restarted")
    } catch (err) {
      const msg = String(err)
      set({ error: msg })
      toast.error("Failed to restart Nginx", { description: msg })
    }
  },

  reloadNginx: async () => {
    try {
      await tauri.nginxReload()
    } catch (err) {
      set({ error: String(err) })
    }
  },
}))
