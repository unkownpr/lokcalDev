import { create } from "zustand"
import { toast } from "sonner"
import type { CertificateInfo, DnsEntry, ResolverStatus } from "@/types/ssl"
import * as tauri from "@/lib/tauri"

interface SslStore {
  mkcertInstalled: boolean
  caInstalled: boolean
  certificates: CertificateInfo[]
  dnsEntries: DnsEntry[]
  resolverStatus: ResolverStatus | null
  loading: boolean
  resolverLoading: boolean
  error: string | null
  checkStatus: () => Promise<void>
  installMkcert: () => Promise<void>
  installCa: () => Promise<void>
  generateCertificate: (domain: string) => Promise<void>
  removeCertificate: (domain: string) => Promise<void>
  fetchCertificates: () => Promise<void>
  addDnsEntry: (domain: string, ip: string) => Promise<void>
  removeDnsEntry: (domain: string) => Promise<void>
  fetchDnsEntries: () => Promise<void>
  fetchResolverStatus: () => Promise<void>
  setupResolver: (tld: string) => Promise<void>
}

export const useSslStore = create<SslStore>((set, get) => ({
  mkcertInstalled: false,
  caInstalled: false,
  certificates: [],
  dnsEntries: [],
  resolverStatus: null,
  loading: false,
  resolverLoading: false,
  error: null,

  checkStatus: async () => {
    try {
      const [mkcertInstalled, caInstalled] = await Promise.all([
        tauri.sslIsMkcertInstalled(),
        tauri.sslIsCaInstalled(),
      ])
      set({ mkcertInstalled, caInstalled })
    } catch (err) {
      set({ error: String(err) })
    }
  },

  installMkcert: async () => {
    set({ loading: true, error: null })
    try {
      await tauri.sslInstallMkcert()
      set({ mkcertInstalled: true, loading: false })
      toast.success("mkcert installed successfully")
    } catch (err) {
      const msg = String(err)
      set({ error: msg, loading: false })
      toast.error("Failed to install mkcert", { description: msg })
    }
  },

  installCa: async () => {
    set({ loading: true, error: null })
    try {
      await tauri.sslInstallCa()
      set({ caInstalled: true, loading: false })
    } catch (err) {
      set({ error: String(err), loading: false })
    }
  },

  generateCertificate: async (domain: string) => {
    try {
      await tauri.sslGenerateCertificate(domain)
      await get().fetchCertificates()
    } catch (err) {
      set({ error: String(err) })
    }
  },

  removeCertificate: async (domain: string) => {
    try {
      await tauri.sslRemoveCertificate(domain)
      await get().fetchCertificates()
    } catch (err) {
      set({ error: String(err) })
    }
  },

  fetchCertificates: async () => {
    try {
      const certificates = await tauri.sslListCertificates()
      set({ certificates })
    } catch (err) {
      set({ error: String(err) })
    }
  },

  addDnsEntry: async (domain: string, ip: string) => {
    try {
      await tauri.dnsAddEntry(domain, ip)
      await get().fetchDnsEntries()
    } catch (err) {
      set({ error: String(err) })
    }
  },

  removeDnsEntry: async (domain: string) => {
    try {
      await tauri.dnsRemoveEntry(domain)
      await get().fetchDnsEntries()
    } catch (err) {
      set({ error: String(err) })
    }
  },

  fetchDnsEntries: async () => {
    try {
      const dnsEntries = await tauri.dnsListEntries()
      set({ dnsEntries })
    } catch (err) {
      set({ error: String(err) })
    }
  },

  fetchResolverStatus: async () => {
    try {
      const resolverStatus = await tauri.dnsGetResolverStatus("test")
      set({ resolverStatus })
    } catch (err) {
      set({ error: String(err) })
    }
  },

  setupResolver: async (tld: string) => {
    set({ resolverLoading: true })
    try {
      await tauri.dnsSetupResolver(tld)
      await get().fetchResolverStatus()
      set({ resolverLoading: false })
      toast.success("DNS resolver configured", {
        description: `All .${tld} domains now resolve to 127.0.0.1 automatically.`,
      })
    } catch (err) {
      set({ error: String(err), resolverLoading: false })
      toast.error("Failed to setup DNS resolver", { description: String(err) })
    }
  },
}))
