import { useEffect, useState } from "react"
import { open } from "@tauri-apps/plugin-shell"
import { Database, Play, Square, RotateCw, Trash2, Plus, ExternalLink } from "lucide-react"
import { Card } from "@/components/ui/card"
import { Button } from "@/components/ui/button"
import { Badge } from "@/components/ui/badge"
import { Input } from "@/components/ui/input"
import {
  Dialog,
  DialogContent,
  DialogHeader,
  DialogTitle,
  DialogTrigger,
} from "@/components/ui/dialog"
import { PageHeader } from "@/components/shared/PageHeader"
import { DownloadButton } from "@/components/shared/DownloadButton"
import { StatusIndicator } from "@/components/layout/StatusIndicator"
import { useDatabaseStore } from "@/stores/databaseStore"
import { useDownloadProgress } from "@/hooks/useDownloadProgress"
import { toast } from "sonner"
import type { PhpMyAdminInfo } from "@/types/database"
import * as tauri from "@/lib/tauri"

export function DatabasePage() {
  const {
    info,
    databases,
    loading,
    fetchInfo,
    install,
    initialize,
    start,
    stop,
    restart,
    fetchDatabases,
    createDatabase,
    dropDatabase,
  } = useDatabaseStore()

  const progress = useDownloadProgress()
  const [newDbName, setNewDbName] = useState("")
  const [dialogOpen, setDialogOpen] = useState(false)
  const [pmaInfo, setPmaInfo] = useState<PhpMyAdminInfo | null>(null)
  const [pmaLoading, setPmaLoading] = useState(false)

  const fetchPmaInfo = async () => {
    try {
      const info = await tauri.phpmyadminGetInfo()
      setPmaInfo(info)
    } catch {
      // ignore
    }
  }

  useEffect(() => {
    fetchInfo()
    fetchPmaInfo()
  }, [fetchInfo])

  useEffect(() => {
    if (info?.running) {
      fetchDatabases()
    }
  }, [info?.running, fetchDatabases])

  const handlePmaInstall = async () => {
    setPmaLoading(true)
    try {
      await tauri.phpmyadminInstall()
      await fetchPmaInfo()
      toast.success("phpMyAdmin installed successfully")
    } catch (err) {
      toast.error("Failed to install phpMyAdmin", { description: String(err) })
    } finally {
      setPmaLoading(false)
    }
  }

  const handleCreate = async () => {
    if (!newDbName) return
    await createDatabase(newDbName)
    setNewDbName("")
    setDialogOpen(false)
  }

  return (
    <div>
      <PageHeader title="Database" description="Manage MariaDB databases">
        {info?.running && (
          <Dialog open={dialogOpen} onOpenChange={setDialogOpen}>
            <DialogTrigger asChild>
              <Button size="sm">
                <Plus className="mr-1.5 h-3.5 w-3.5" />
                Create Database
              </Button>
            </DialogTrigger>
            <DialogContent>
              <DialogHeader>
                <DialogTitle>Create Database</DialogTitle>
              </DialogHeader>
              <div className="space-y-4 pt-2">
                <Input
                  placeholder="database_name"
                  value={newDbName}
                  onChange={(e) => setNewDbName(e.target.value)}
                />
                <Button className="w-full" onClick={handleCreate} disabled={!newDbName}>
                  Create
                </Button>
              </div>
            </DialogContent>
          </Dialog>
        )}
      </PageHeader>

      {/* MariaDB Status Card */}
      <Card className="p-4 mb-6">
        <div className="flex items-center justify-between mb-3">
          <div className="flex items-center gap-2">
            <StatusIndicator status={info?.running ? "running" : "stopped"} />
            <span className="font-medium text-sm">MariaDB</span>
            {info?.version && (
              <Badge variant="outline" className="text-[10px]">{info.version}</Badge>
            )}
          </div>
          <Badge variant={info?.installed ? (info?.running ? "default" : "secondary") : "outline"}>
            {!info?.installed
              ? "Not Installed"
              : !info?.initialized
                ? "Not Initialized"
                : info?.running
                  ? "Running"
                  : "Stopped"}
          </Badge>
        </div>

        {info?.installed && (
          <div className="text-xs text-muted-foreground mb-3 space-y-0.5">
            <p>Port: {info.port}</p>
            {info.pid && <p>PID: {info.pid}</p>}
          </div>
        )}

        <div className="flex gap-2">
          {!info?.installed ? (
            <DownloadButton
              label="Install MariaDB"
              progress={progress["mariadb"]}
              onClick={install}
              disabled={loading}
            />
          ) : !info?.initialized ? (
            <Button size="sm" variant="outline" onClick={initialize} disabled={loading}>
              Initialize Database
            </Button>
          ) : (
            <>
              {!info.running ? (
                <Button size="sm" variant="outline" onClick={start}>
                  <Play className="mr-1 h-3 w-3" /> Start
                </Button>
              ) : (
                <Button size="sm" variant="outline" onClick={stop}>
                  <Square className="mr-1 h-3 w-3" /> Stop
                </Button>
              )}
              {info.running && (
                <Button size="sm" variant="outline" onClick={restart}>
                  <RotateCw className="mr-1 h-3 w-3" /> Restart
                </Button>
              )}
            </>
          )}
        </div>
      </Card>

      {/* phpMyAdmin Card */}
      <Card className="p-4 mb-6">
        <div className="flex items-center justify-between mb-3">
          <div className="flex items-center gap-2">
            <StatusIndicator status={pmaInfo?.installed ? "running" : "stopped"} />
            <span className="font-medium text-sm">phpMyAdmin</span>
            {pmaInfo?.version && (
              <Badge variant="outline" className="text-[10px]">{pmaInfo.version}</Badge>
            )}
          </div>
          <Badge variant={pmaInfo?.installed ? "default" : "outline"}>
            {pmaInfo?.installed ? "Installed" : "Not Installed"}
          </Badge>
        </div>

        <div className="flex gap-2">
          {!pmaInfo?.installed ? (
            <DownloadButton
              label="Install phpMyAdmin"
              progress={progress["phpmyadmin"]}
              onClick={handlePmaInstall}
              disabled={pmaLoading}
            />
          ) : (
            <Button
              size="sm"
              variant="outline"
              onClick={() => open("http://localhost/phpmyadmin")}
            >
              <ExternalLink className="mr-1 h-3 w-3" /> Open phpMyAdmin
            </Button>
          )}
        </div>
      </Card>

      {/* Database List */}
      {info?.running && (
        <div>
          <h3 className="text-sm font-medium mb-3">Databases</h3>
          {databases.length === 0 ? (
            <p className="text-sm text-muted-foreground">No databases found.</p>
          ) : (
            <div className="space-y-1">
              {databases.map((db) => (
                <div
                  key={db.name}
                  className="flex items-center justify-between py-2 px-3 rounded hover:bg-muted/50"
                >
                  <div className="flex items-center gap-2">
                    <Database className="h-3.5 w-3.5 text-muted-foreground" />
                    <span className="text-sm font-mono">{db.name}</span>
                  </div>
                  {!["mysql", "information_schema", "performance_schema", "sys"].includes(db.name) && (
                    <Button
                      size="sm"
                      variant="ghost"
                      className="text-destructive h-7"
                      onClick={() => dropDatabase(db.name)}
                    >
                      <Trash2 className="h-3 w-3" />
                    </Button>
                  )}
                </div>
              ))}
            </div>
          )}
        </div>
      )}
    </div>
  )
}
