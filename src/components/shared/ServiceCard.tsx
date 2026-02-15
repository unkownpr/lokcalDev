import { Play, Square, RotateCw, Download, Database, ExternalLink } from "lucide-react"
import { open } from "@tauri-apps/plugin-shell"
import { Button } from "@/components/ui/button"
import { Card } from "@/components/ui/card"
import { Badge } from "@/components/ui/badge"
import { StatusIndicator } from "@/components/layout/StatusIndicator"
import type { ServiceInfo } from "@/types/service"

interface ServiceCardProps {
  service: ServiceInfo
  onInstall: (id: string) => void
  onInitialize: (id: string) => void
  onStart: (id: string) => void
  onStop: (id: string) => void
  onRestart: (id: string) => void
}

export function ServiceCard({ service, onInstall, onInitialize, onStart, onStop, onRestart }: ServiceCardProps) {
  const isRunning = service.status === "running"
  const isWebOnly = service.id === "phpmyadmin"

  return (
    <Card className="flex items-center justify-between p-4">
      <div className="flex items-center gap-3">
        <StatusIndicator
          status={!service.installed ? "stopped" : isWebOnly ? (service.installed ? "running" : "stopped") : service.status}
          size="md"
        />
        <div>
          <div className="flex items-center gap-2">
            <p className="text-sm font-medium text-foreground">{service.name}</p>
            {!service.installed && (
              <Badge variant="outline" className="text-[10px]">Not Installed</Badge>
            )}
            {service.installed && !service.initialized && !isWebOnly && (
              <Badge variant="outline" className="text-[10px]">Not Initialized</Badge>
            )}
          </div>
          <p className="text-[11px] text-muted-foreground">
            {service.version && `v${service.version}`}
            {service.port && ` · Port ${service.port}`}
            {isRunning && service.pid && ` · PID ${service.pid}`}
          </p>
        </div>
      </div>
      <div className="flex items-center gap-1">
        {!service.installed ? (
          <Button
            variant="outline"
            size="sm"
            onClick={() => onInstall(service.id)}
          >
            <Download className="mr-1.5 h-3.5 w-3.5" />
            Install
          </Button>
        ) : isWebOnly ? (
          <Button
            variant="ghost"
            size="sm"
            onClick={() => open("http://localhost/phpmyadmin")}
          >
            <ExternalLink className="mr-1.5 h-3.5 w-3.5" />
            Open
          </Button>
        ) : !service.initialized ? (
          <Button
            variant="outline"
            size="sm"
            onClick={() => onInitialize(service.id)}
          >
            <Database className="mr-1.5 h-3.5 w-3.5" />
            Initialize
          </Button>
        ) : isRunning ? (
          <>
            <Button
              variant="ghost"
              size="icon"
              className="h-7 w-7"
              onClick={() => onRestart(service.id)}
            >
              <RotateCw className="h-3.5 w-3.5" />
            </Button>
            <Button
              variant="ghost"
              size="icon"
              className="h-7 w-7 text-destructive hover:text-destructive"
              onClick={() => onStop(service.id)}
            >
              <Square className="h-3.5 w-3.5" />
            </Button>
          </>
        ) : (
          <Button
            variant="ghost"
            size="icon"
            className="h-7 w-7 text-emerald-600 hover:text-emerald-600"
            onClick={() => onStart(service.id)}
          >
            <Play className="h-3.5 w-3.5" />
          </Button>
        )}
      </div>
    </Card>
  )
}
