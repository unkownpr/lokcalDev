import { useEffect, useState } from "react"
import { Shield, Trash2, CheckCircle2, XCircle, Download, Loader2, Wifi } from "lucide-react"
import { Card } from "@/components/ui/card"
import { Button } from "@/components/ui/button"
import { Badge } from "@/components/ui/badge"
import { Input } from "@/components/ui/input"
import { Tabs, TabsContent, TabsList, TabsTrigger } from "@/components/ui/tabs"
import { PageHeader } from "@/components/shared/PageHeader"
import { useSslStore } from "@/stores/sslStore"
import { useSiteStore } from "@/stores/siteStore"

export function SslPage() {
  const {
    mkcertInstalled,
    caInstalled,
    certificates,
    dnsEntries,
    resolverStatus,
    loading,
    resolverLoading,
    checkStatus,
    installMkcert,
    installCa,
    generateCertificate,
    removeCertificate,
    fetchCertificates,
    addDnsEntry,
    removeDnsEntry,
    fetchDnsEntries,
    fetchResolverStatus,
    setupResolver,
  } = useSslStore()

  const { sites, fetchSites } = useSiteStore()

  const [newDomain, setNewDomain] = useState("")
  const [dnsDomain, setDnsDomain] = useState("")
  const [dnsIp, setDnsIp] = useState("127.0.0.1")

  useEffect(() => {
    checkStatus()
    fetchCertificates()
    fetchDnsEntries()
    fetchResolverStatus()
    fetchSites()
  }, [checkStatus, fetchCertificates, fetchDnsEntries, fetchResolverStatus, fetchSites])

  return (
    <div>
      <PageHeader title="SSL" description="Manage SSL certificates and DNS for local sites" />

      <Tabs defaultValue="dns">
        <TabsList>
          <TabsTrigger value="dns">DNS</TabsTrigger>
          <TabsTrigger value="setup">SSL Setup</TabsTrigger>
          <TabsTrigger value="certificates">Certificates</TabsTrigger>
        </TabsList>

        <TabsContent value="dns" className="mt-4 space-y-4">
          {/* Auto DNS Resolver Card */}
          <Card className="p-4">
            <div className="flex items-center justify-between mb-3">
              <div className="flex items-center gap-2">
                <Wifi className="h-4 w-4 text-muted-foreground" />
                <div>
                  <p className="text-sm font-medium">Auto DNS Resolver</p>
                  <p className="text-xs text-muted-foreground">
                    Automatically resolve all .test domains to 127.0.0.1 via dnsmasq
                  </p>
                </div>
              </div>
              <Badge variant={resolverStatus?.configured ? "default" : "secondary"}>
                {resolverStatus?.configured ? "Active" : "Not Configured"}
              </Badge>
            </div>

            {resolverStatus?.configured ? (
              <div className="text-xs text-muted-foreground space-y-0.5">
                <p>All .test domains automatically resolve to 127.0.0.1</p>
                <p>No need to manually add DNS entries for .test sites.</p>
              </div>
            ) : (
              <div className="space-y-3">
                <p className="text-xs text-muted-foreground">
                  Setup dnsmasq to auto-resolve all .test domains. This is a one-time setup
                  that requires your password once. After that, all .test sites work automatically
                  without any password prompts.
                </p>

                {resolverStatus && !resolverStatus.configured && (
                  <div className="text-xs text-muted-foreground space-y-0.5">
                    {resolverStatus.dnsmasqInstalled ? (
                      <p className="flex items-center gap-1"><CheckCircle2 className="h-3 w-3 text-emerald-500" /> dnsmasq installed</p>
                    ) : (
                      <p className="flex items-center gap-1"><XCircle className="h-3 w-3" /> dnsmasq not installed (will be installed via Homebrew)</p>
                    )}
                    {resolverStatus.dnsmasqRunning ? (
                      <p className="flex items-center gap-1"><CheckCircle2 className="h-3 w-3 text-emerald-500" /> dnsmasq running</p>
                    ) : (
                      <p className="flex items-center gap-1"><XCircle className="h-3 w-3" /> dnsmasq not running</p>
                    )}
                    {resolverStatus.resolverExists ? (
                      <p className="flex items-center gap-1"><CheckCircle2 className="h-3 w-3 text-emerald-500" /> Resolver file exists</p>
                    ) : (
                      <p className="flex items-center gap-1"><XCircle className="h-3 w-3" /> Resolver file missing</p>
                    )}
                  </div>
                )}

                <Button
                  size="sm"
                  onClick={() => setupResolver("test")}
                  disabled={resolverLoading}
                >
                  {resolverLoading ? (
                    <>
                      <Loader2 className="mr-1.5 h-3.5 w-3.5 animate-spin" />
                      Setting up...
                    </>
                  ) : (
                    "Setup Auto DNS Resolver"
                  )}
                </Button>
              </div>
            )}
          </Card>

          {/* Manual DNS Entry */}
          <Card className="p-4">
            <h3 className="text-sm font-medium mb-3">Manual DNS Entry</h3>
            <p className="text-xs text-muted-foreground mb-3">
              Add individual entries to /etc/hosts. Not needed if Auto DNS Resolver is active for .test domains.
            </p>
            <div className="flex gap-2">
              <Input
                placeholder="mysite.test"
                value={dnsDomain}
                onChange={(e) => setDnsDomain(e.target.value)}
                className="flex-1"
              />
              <Input
                placeholder="127.0.0.1"
                value={dnsIp}
                onChange={(e) => setDnsIp(e.target.value)}
                className="w-36"
              />
              <Button
                size="sm"
                onClick={() => {
                  if (dnsDomain && dnsIp) {
                    addDnsEntry(dnsDomain, dnsIp)
                    setDnsDomain("")
                  }
                }}
                disabled={!dnsDomain || !dnsIp}
              >
                Add
              </Button>
            </div>
          </Card>

          {/* DNS Entries — dnsmasq aktifse sitelerden, değilse /etc/hosts'tan */}
          {resolverStatus?.configured ? (
            <div>
              <h3 className="text-sm font-medium mb-3">Managed Domains</h3>
              {sites.filter(s => s.active).length === 0 ? (
                <p className="text-xs text-muted-foreground">No active sites yet.</p>
              ) : (
                <div className="space-y-1">
                  {sites.filter(s => s.active).map((site) => (
                    <div
                      key={site.id}
                      className="flex items-center justify-between py-2 px-3 rounded hover:bg-muted/50"
                    >
                      <div className="flex items-center gap-3">
                        <span className="text-sm font-mono">{site.domain}</span>
                        <span className="text-xs text-muted-foreground">127.0.0.1</span>
                      </div>
                      <Badge variant="secondary" className="text-[10px]">Auto DNS</Badge>
                    </div>
                  ))}
                </div>
              )}
            </div>
          ) : dnsEntries.length > 0 ? (
            <div>
              <h3 className="text-sm font-medium mb-3">DNS Entries (/etc/hosts)</h3>
              <div className="space-y-1">
                {dnsEntries.map((entry) => (
                  <div
                    key={entry.domain}
                    className="flex items-center justify-between py-2 px-3 rounded hover:bg-muted/50"
                  >
                    <div className="flex items-center gap-3">
                      <span className="text-sm font-mono">{entry.domain}</span>
                      <span className="text-xs text-muted-foreground">{entry.ip}</span>
                    </div>
                    <Button
                      size="sm"
                      variant="ghost"
                      className="text-destructive h-7"
                      onClick={() => removeDnsEntry(entry.domain)}
                    >
                      <Trash2 className="h-3 w-3" />
                    </Button>
                  </div>
                ))}
              </div>
            </div>
          ) : null}
        </TabsContent>

        <TabsContent value="setup" className="mt-4 space-y-4">
          <Card className="p-4">
            <div className="flex items-center justify-between">
              <div className="flex items-center gap-2">
                {mkcertInstalled ? (
                  <CheckCircle2 className="h-4 w-4 text-emerald-500" />
                ) : (
                  <XCircle className="h-4 w-4 text-muted-foreground" />
                )}
                <div>
                  <p className="text-sm font-medium">mkcert</p>
                  <p className="text-xs text-muted-foreground">Local certificate authority tool</p>
                </div>
              </div>
              {!mkcertInstalled && (
                <Button size="sm" variant="outline" onClick={installMkcert} disabled={loading}>
                  <Download className="mr-1.5 h-3.5 w-3.5" /> Install
                </Button>
              )}
              {mkcertInstalled && <Badge>Installed</Badge>}
            </div>
          </Card>

          <Card className="p-4">
            <div className="flex items-center justify-between">
              <div className="flex items-center gap-2">
                {caInstalled ? (
                  <CheckCircle2 className="h-4 w-4 text-emerald-500" />
                ) : (
                  <XCircle className="h-4 w-4 text-muted-foreground" />
                )}
                <div>
                  <p className="text-sm font-medium">Root CA</p>
                  <p className="text-xs text-muted-foreground">Trusted root certificate authority</p>
                </div>
              </div>
              {!caInstalled && mkcertInstalled && (
                <Button size="sm" variant="outline" onClick={installCa} disabled={loading}>
                  Install CA
                </Button>
              )}
              {caInstalled && <Badge>Installed</Badge>}
            </div>
          </Card>
        </TabsContent>

        <TabsContent value="certificates" className="mt-4">
          <Card className="p-4 mb-4">
            <h3 className="text-sm font-medium mb-3">Generate Certificate</h3>
            <div className="flex gap-2">
              <Input
                placeholder="mysite.test"
                value={newDomain}
                onChange={(e) => setNewDomain(e.target.value)}
                className="flex-1"
              />
              <Button
                size="sm"
                onClick={() => {
                  if (newDomain) {
                    generateCertificate(newDomain)
                    setNewDomain("")
                  }
                }}
                disabled={!newDomain || !mkcertInstalled || !caInstalled}
              >
                Generate
              </Button>
            </div>
          </Card>

          {certificates.length === 0 ? (
            <p className="text-sm text-muted-foreground">No certificates generated yet.</p>
          ) : (
            <div className="space-y-1">
              {certificates.map((cert) => (
                <div
                  key={cert.domain}
                  className="flex items-center justify-between py-2 px-3 rounded hover:bg-muted/50"
                >
                  <div className="flex items-center gap-2">
                    <Shield className="h-3.5 w-3.5 text-muted-foreground" />
                    <span className="text-sm font-mono">{cert.domain}</span>
                    {cert.exists && (
                      <Badge variant="secondary" className="text-[10px]">valid</Badge>
                    )}
                  </div>
                  <Button
                    size="sm"
                    variant="ghost"
                    className="text-destructive h-7"
                    onClick={() => removeCertificate(cert.domain)}
                  >
                    <Trash2 className="h-3 w-3" />
                  </Button>
                </div>
              ))}
            </div>
          )}
        </TabsContent>
      </Tabs>
    </div>
  )
}
