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
  createSite: (name: string, domain: string, documentRoot: string, phpVersion: string, ssl: boolean, template?: string) => Promise<void>
  setupTemplate: (siteId: string, template: string) => Promise<void>
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

  createSite: async (name, domain, documentRoot, phpVersion, ssl, template) => {
    let site: Site | null = null
    try {
      site = await tauri.siteCreate(name, domain, documentRoot, phpVersion, ssl, template)
    } catch (err) {
      const msg = String(err)
      set({ error: msg })
      toast.error("Failed to create site", { description: msg })
      return
    }

    // Privileged operations (require admin password on macOS)
    let dnsOk = true
    let nginxOk = true

    // Add DNS entry
    try {
      await tauri.dnsAddEntry(domain, "127.0.0.1")
    } catch {
      dnsOk = false
    }

    // Generate SSL certificate if enabled
    if (ssl) {
      try {
        await tauri.sslGenerateCertificate(domain)
      } catch {}
    }

    // Reload nginx to pick up new config
    try {
      await tauri.nginxReload()
    } catch {
      nginxOk = false
    }

    // If both privileged operations failed (user likely cancelled), roll back
    if (!dnsOk && !nginxOk) {
      try {
        await tauri.siteDelete(site.id)
      } catch {}
      await get().fetchSites()
      toast.error("Site creation cancelled", {
        description: "Admin access is required for DNS and Nginx configuration.",
      })
      return
    }

    await get().fetchSites()
    if (dnsOk && nginxOk) {
      toast.success(`Site "${name}" created`, { description: `${ssl ? "https" : "http"}://${domain}` })
    } else {
      toast.warning(`Site "${name}" created with warnings`, {
        description: "Some operations required admin access and were skipped.",
      })
    }

    // Fire template setup in background if requested
    if (template && site) {
      get().setupTemplate(site.id, template)
    }
  },

  setupTemplate: async (siteId, template) => {
    try {
      await tauri.siteSetupTemplate(siteId, template)
      await get().fetchSites()
      toast.success(`${template.charAt(0).toUpperCase() + template.slice(1)} installed successfully`)
    } catch (err) {
      await get().fetchSites()
      toast.error(`${template.charAt(0).toUpperCase() + template.slice(1)} setup failed`, {
        description: String(err),
      })
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

    let hasWarnings = false

    // Handle domain change: remove old DNS, add new DNS
    if (data.domain && oldSite && data.domain !== oldSite.domain) {
      try {
        await tauri.dnsRemoveEntry(oldSite.domain)
      } catch {
        hasWarnings = true
      }
      try {
        await tauri.dnsAddEntry(data.domain, "127.0.0.1")
      } catch {
        hasWarnings = true
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
      } catch {
        hasWarnings = true
      }
    }

    // Reload nginx
    try {
      await tauri.nginxReload()
    } catch {
      hasWarnings = true
    }

    await get().fetchSites()
    if (!hasWarnings) {
      toast.success("Site updated")
    } else {
      toast.warning("Site updated with warnings", {
        description: "Some operations required admin access and were skipped.",
      })
    }
  },

  deleteSite: async (id) => {
    const site = get().sites.find((s) => s.id === id)

    // First, try privileged operations (require admin password on macOS)
    let dnsOk = true
    let nginxOk = true

    if (site) {
      try {
        await tauri.dnsRemoveEntry(site.domain)
      } catch {
        dnsOk = false
      }
    }

    // If DNS failed (user cancelled password), skip the rest
    if (!dnsOk) {
      toast.error("Site deletion cancelled", {
        description: "Admin access is required to update DNS and Nginx.",
      })
      return
    }

    // Delete site files + nginx config
    try {
      await tauri.siteDelete(id)
    } catch (err) {
      const msg = String(err)
      set({ error: msg })
      toast.error("Failed to delete site", { description: msg })
      return
    }

    // Reload nginx
    try {
      await tauri.nginxReload()
    } catch {
      nginxOk = false
    }

    await get().fetchSites()
    if (nginxOk) {
      toast.success("Site deleted")
    } else {
      toast.warning("Site deleted", {
        description: "Nginx reload was skipped — admin access required.",
      })
    }
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
