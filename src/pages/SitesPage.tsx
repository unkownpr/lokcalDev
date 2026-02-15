import { useEffect } from "react"
import { Globe, Trash2, ExternalLink } from "lucide-react"
import { open } from "@tauri-apps/plugin-shell"
import { Card } from "@/components/ui/card"
import { Button } from "@/components/ui/button"
import { Badge } from "@/components/ui/badge"
import { PageHeader } from "@/components/shared/PageHeader"
import { EmptyState } from "@/components/shared/EmptyState"
import { SiteCreateDialog } from "@/components/shared/SiteCreateDialog"
import { StatusIndicator } from "@/components/layout/StatusIndicator"
import { useSiteStore } from "@/stores/siteStore"
import { usePhpStore } from "@/stores/phpStore"

export function SitesPage() {
  const { sites, fetchSites, createSite, deleteSite } = useSiteStore()
  const { versions, fetchVersions } = usePhpStore()

  useEffect(() => {
    fetchSites()
    fetchVersions()
  }, [fetchSites, fetchVersions])

  const installedPhpVersions = versions.filter((v) => v.installed).map((v) => v.version)

  return (
    <div>
      <PageHeader title="Sites" description="Manage your local websites">
        <SiteCreateDialog
          phpVersions={installedPhpVersions}
          onSubmit={async (name, domain, documentRoot, phpVersion, ssl) => {
            await createSite(name, domain, documentRoot, phpVersion, ssl)
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
        <div className="space-y-3">
          {sites.map((site) => (
            <Card key={site.id} className="p-4">
              <div className="flex items-center justify-between">
                <div className="flex items-center gap-3">
                  <StatusIndicator status={site.active ? "running" : "stopped"} />
                  <div>
                    <div className="flex items-center gap-2">
                      <span className="text-sm font-medium">{site.name}</span>
                      <Badge variant="outline" className="text-[10px]">
                        PHP {site.phpVersion}
                      </Badge>
                      {site.ssl && (
                        <Badge variant="secondary" className="text-[10px]">
                          SSL
                        </Badge>
                      )}
                    </div>
                    <p className="text-xs text-muted-foreground mt-0.5">
                      {site.ssl ? "https" : "http"}://{site.domain}
                    </p>
                  </div>
                </div>
                <div className="flex items-center gap-2">
                  <span className="text-xs text-muted-foreground max-w-[200px] truncate">
                    {site.documentRoot}
                  </span>
                  <Button
                    size="sm"
                    variant="ghost"
                    onClick={() => open(`${site.ssl ? "https" : "http"}://${site.domain}`)}
                  >
                    <ExternalLink className="h-3.5 w-3.5" />
                  </Button>
                  <Button
                    size="sm"
                    variant="ghost"
                    className="text-destructive"
                    onClick={() => deleteSite(site.id)}
                  >
                    <Trash2 className="h-3.5 w-3.5" />
                  </Button>
                </div>
              </div>
            </Card>
          ))}
        </div>
      )}
    </div>
  )
}
