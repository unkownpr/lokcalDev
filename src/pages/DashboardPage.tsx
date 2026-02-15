import { useEffect } from "react"
import { Activity, Globe, Server, Database } from "lucide-react"
import { Card } from "@/components/ui/card"
import { PageHeader } from "@/components/shared/PageHeader"
import { StatusIndicator } from "@/components/layout/StatusIndicator"
import { useServiceStore } from "@/stores/serviceStore"
import { useAppStore } from "@/stores/appStore"
import { useSiteStore } from "@/stores/siteStore"
import { useDatabaseStore } from "@/stores/databaseStore"

export function DashboardPage() {
  const services = useServiceStore((s) => s.services)
  const systemInfo = useAppStore((s) => s.systemInfo)
  const sites = useSiteStore((s) => s.sites)
  const fetchSites = useSiteStore((s) => s.fetchSites)
  const databases = useDatabaseStore((s) => s.databases)
  const dbInfo = useDatabaseStore((s) => s.info)
  const fetchDbInfo = useDatabaseStore((s) => s.fetchInfo)
  const fetchDatabases = useDatabaseStore((s) => s.fetchDatabases)

  useEffect(() => {
    fetchSites()
    fetchDbInfo()
  }, [fetchSites, fetchDbInfo])

  useEffect(() => {
    if (dbInfo?.running) {
      fetchDatabases()
    }
  }, [dbInfo?.running, fetchDatabases])

  const runningServices = services.filter((s) => s.status === "running").length

  const summaryCards = [
    { label: "Active Sites", value: String(sites.filter((s) => s.active).length), icon: Globe },
    { label: "Services", value: `${runningServices}/${services.length}`, icon: Server },
    { label: "Databases", value: String(dbInfo?.running ? databases.length : 0), icon: Database },
    { label: "Uptime", value: runningServices > 0 ? "Active" : "--", icon: Activity },
  ]

  return (
    <div>
      <PageHeader
        title="Dashboard"
        description="Overview of your local development environment"
      />

      <div className="grid grid-cols-4 gap-4 mb-6">
        {summaryCards.map((card) => (
          <Card key={card.label} className="p-4">
            <div className="flex items-center justify-between">
              <div>
                <p className="text-[11px] font-medium text-muted-foreground">
                  {card.label}
                </p>
                <p className="mt-1 text-2xl font-semibold tracking-tight text-foreground">
                  {card.value}
                </p>
              </div>
              <div className="rounded-lg bg-muted p-2">
                <card.icon className="h-4 w-4 text-muted-foreground" />
              </div>
            </div>
          </Card>
        ))}
      </div>

      <div className="grid grid-cols-2 gap-4">
        <Card className="p-4">
          <h3 className="mb-3 text-sm font-medium text-foreground">
            Services
          </h3>
          <div className="space-y-2.5">
            {services.map((service) => (
              <div key={service.id} className="flex items-center justify-between">
                <div className="flex items-center gap-2">
                  <StatusIndicator status={service.status} />
                  <span className="text-sm text-foreground">{service.name}</span>
                </div>
                <span className="text-[11px] text-muted-foreground capitalize">
                  {service.status}
                </span>
              </div>
            ))}
          </div>
        </Card>

        <Card className="p-4">
          <h3 className="mb-3 text-sm font-medium text-foreground">
            System Info
          </h3>
          {systemInfo ? (
            <div className="space-y-2">
              {[
                { label: "Platform", value: systemInfo.os },
                { label: "Architecture", value: systemInfo.arch },
                { label: "Version", value: `v${systemInfo.appVersion}` },
                { label: "Data Directory", value: systemInfo.dataDir },
              ].map((row) => (
                <div key={row.label} className="flex items-center justify-between">
                  <span className="text-sm text-muted-foreground">
                    {row.label}
                  </span>
                  <span className="text-sm text-foreground truncate max-w-[200px]">
                    {row.value}
                  </span>
                </div>
              ))}
            </div>
          ) : (
            <p className="text-sm text-muted-foreground">Loading...</p>
          )}
        </Card>
      </div>
    </div>
  )
}
