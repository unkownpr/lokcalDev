import { useEffect } from "react"
import { Globe, Trash2, ExternalLink, RotateCcw, Loader2 } from "lucide-react"
import { open } from "@tauri-apps/plugin-shell"
import { toast } from "sonner"
import { Card } from "@/components/ui/card"
import { Button } from "@/components/ui/button"
import { Badge } from "@/components/ui/badge"
import { PageHeader } from "@/components/shared/PageHeader"
import { EmptyState } from "@/components/shared/EmptyState"
import { SiteCreateDialog } from "@/components/shared/SiteCreateDialog"
import { StatusIndicator } from "@/components/layout/StatusIndicator"
import { ConfirmDialog } from "@/components/shared/ConfirmDialog"
import { useSiteStore } from "@/stores/siteStore"
import { usePhpStore } from "@/stores/phpStore"
import { useServiceStore } from "@/stores/serviceStore"

export function SitesPage() {
  const { sites, nginxInfo, fetchSites, fetchNginxInfo, createSite, deleteSite, setupTemplate } = useSiteStore()
  const { versions, fetchVersions } = usePhpStore()
  const { services, fetchServices } = useServiceStore()

  useEffect(() => {
    fetchSites()
    fetchNginxInfo()
    fetchVersions()
    fetchServices()
  }, [fetchSites, fetchNginxInfo, fetchVersions, fetchServices])

  const nginxRunning = services.find((s) => s.id === "nginx")?.status === "running"
  const nginxPort = nginxInfo?.port ?? 8080
  const nginxSslPort = nginxInfo?.sslPort ?? 8443

  const isPhpFpmRunning = (phpVersion: string) => {
    return services.find((s) => s.id === `php-fpm-${phpVersion}`)?.status === "running"
  }

  // Check which PHP versions are needed but not running
  const stoppedPhpVersions = [...new Set(sites.filter((s) => s.active).map((s) => s.phpVersion))]
    .filter((v) => services.find((s) => s.id === `php-fpm-${v}`)?.status !== "running")

  const handleOpenSite = (domain: string, ssl: boolean, phpVersion: string) => {
    if (!nginxRunning) {
      toast.error("Nginx is not running", { description: "Start Nginx from the Services page first." })
      return
    }
    if (!isPhpFpmRunning(phpVersion)) {
      toast.error(`PHP-FPM ${phpVersion} is not running`, {
        description: "Start it from the Services page or install PHP from the PHP page.",
      })
      return
    }
    open(getSiteUrl(domain, ssl))
  }

  const getSiteUrl = (domain: string, ssl: boolean) => {
    if (ssl) {
      const portSuffix = nginxSslPort === 443 ? "" : `:${nginxSslPort}`
      return `https://${domain}${portSuffix}`
    }
    const portSuffix = nginxPort === 80 ? "" : `:${nginxPort}`
    return `http://${domain}${portSuffix}`
  }

  const installedPhpVersions = versions.filter((v) => v.installed).map((v) => v.version)

  return (
    <div>
      <PageHeader title="Sites" description="Manage your local websites">
        <SiteCreateDialog
          phpVersions={installedPhpVersions}
          onSubmit={async (name, domain, documentRoot, phpVersion, ssl, template) => {
            await createSite(name, domain, documentRoot, phpVersion, ssl, template)
          }}
        />
      </PageHeader>

      {sites.length === 0 ? (
        <EmptyState
          icon={Globe}
          title="No sites yet"
          description="Add your first site to start developing locally with custom domains and SSL."
        />
      ) : (
        <div>
          {!nginxRunning && (
            <div className="mb-4 rounded-md border border-border bg-muted/50 px-4 py-2.5 text-xs text-muted-foreground">
              Start Nginx first for sites to be accessible.
            </div>
          )}
          {nginxRunning && stoppedPhpVersions.length > 0 && (
            <div className="mb-4 rounded-md border border-amber-500/30 bg-amber-500/5 px-4 py-2.5 text-xs text-amber-700 dark:text-amber-400">
              PHP-FPM {stoppedPhpVersions.join(", ")} is not running. Sites using {stoppedPhpVersions.length > 1 ? "these versions" : "this version"} will show 502 errors.
            </div>
          )}
          <div className="space-y-3">
          {sites.map((site) => (
            <Card key={site.id} className="p-4">
              <div className="flex items-center justify-between">
                <div className="flex items-center gap-3">
                  <StatusIndicator status={site.active && nginxRunning && isPhpFpmRunning(site.phpVersion) ? "running" : "stopped"} />
                  <div>
                    <div className="flex items-center gap-2">
                      <span className="text-sm font-medium">{site.name}</span>
                      {site.template && (
                        <Badge variant="default" className="text-[10px]">
                          {site.template === "wordpress" ? "WordPress" : site.template === "laravel" ? "Laravel" : "Fat-Free"}
                        </Badge>
                      )}
                      <Badge variant="outline" className="text-[10px]">
                        PHP {site.phpVersion}
                      </Badge>
                      {site.ssl && (
                        <Badge variant="secondary" className="text-[10px]">
                          SSL
                        </Badge>
                      )}
                      {site.templateStatus === "installing" && (
                        <Badge variant="secondary" className="text-[10px] gap-1">
                          <Loader2 className="h-2.5 w-2.5 animate-spin" />
                          Installing...
                        </Badge>
                      )}
                      {site.templateStatus === "pending" && (
                        <Badge variant="secondary" className="text-[10px] gap-1">
                          <Loader2 className="h-2.5 w-2.5 animate-spin" />
                          Pending...
                        </Badge>
                      )}
                      {site.templateStatus === "failed" && (
                        <Badge variant="destructive" className="text-[10px]">
                          Setup Failed
                        </Badge>
                      )}
                    </div>
                    <p className="text-xs text-muted-foreground mt-0.5">
                      {getSiteUrl(site.domain, site.ssl)}
                    </p>
                  </div>
                </div>
                <div className="flex items-center gap-2">
                  <span className="text-xs text-muted-foreground max-w-[200px] truncate">
                    {site.documentRoot}
                  </span>
                  {site.templateStatus === "failed" && site.template && (
                    <Button
                      size="sm"
                      variant="ghost"
                      title="Retry template setup"
                      onClick={() => setupTemplate(site.id, site.template!)}
                    >
                      <RotateCcw className="h-3.5 w-3.5" />
                    </Button>
                  )}
                  <Button
                    size="sm"
                    variant="ghost"
                    onClick={() => handleOpenSite(site.domain, site.ssl, site.phpVersion)}
                  >
                    <ExternalLink className="h-3.5 w-3.5" />
                  </Button>
                  <ConfirmDialog
                    trigger={
                      <Button size="sm" variant="ghost" className="text-destructive">
                        <Trash2 className="h-3.5 w-3.5" />
                      </Button>
                    }
                    title={`Delete "${site.name}"?`}
                    description="This will remove the site configuration, DNS entry, and SSL certificate. Your project files will not be deleted."
                    confirmLabel="Delete"
                    onConfirm={() => deleteSite(site.id)}
                  />
                </div>
              </div>
            </Card>
          ))}
          </div>
        </div>
      )}
    </div>
  )
}
