import { useEffect, useState } from "react"
import { listen } from "@tauri-apps/api/event"
import { Progress } from "@/components/ui/progress"
import { Download, Loader2, CheckCircle2, XCircle } from "lucide-react"
import type { DownloadProgress } from "@/types/php"
import { formatBytes } from "@/lib/utils"

const LABELS: Record<string, string> = {
  "php-8.1": "PHP 8.1",
  "php-8.2": "PHP 8.2",
  "php-8.3": "PHP 8.3",
  "php-8.4": "PHP 8.4",
  nginx: "Nginx",
  mariadb: "MariaDB",
  mkcert: "mkcert",
}

export function GlobalDownloadBar() {
  const [downloads, setDownloads] = useState<Record<string, DownloadProgress>>({})

  useEffect(() => {
    const unlisten = listen<DownloadProgress>("download-progress", (event) => {
      const p = event.payload
      setDownloads((prev) => ({ ...prev, [p.id]: p }))

      // Auto-remove completed/failed downloads after delay
      if (p.status === "completed" || p.status === "failed") {
        setTimeout(() => {
          setDownloads((prev) => {
            const next = { ...prev }
            delete next[p.id]
            return next
          })
        }, p.status === "failed" ? 8000 : 3000)
      }
    })

    return () => {
      unlisten.then((fn) => fn())
    }
  }, [])

  const active = Object.values(downloads)
  if (active.length === 0) return null

  return (
    <div className="border-t bg-muted/30 px-4 py-2 space-y-2">
      {active.map((dl) => {
        const label = LABELS[dl.id] || dl.id
        const isExtracting = dl.status === "extracting"
        const isCompleted = dl.status === "completed"
        const isFailed = dl.status === "failed"

        return (
          <div key={dl.id} className="flex items-center gap-3">
            <div className="flex items-center gap-1.5 min-w-[100px]">
              {isCompleted ? (
                <CheckCircle2 className="h-3.5 w-3.5 text-emerald-500 shrink-0" />
              ) : isFailed ? (
                <XCircle className="h-3.5 w-3.5 text-destructive shrink-0" />
              ) : isExtracting ? (
                <Loader2 className="h-3.5 w-3.5 animate-spin text-muted-foreground shrink-0" />
              ) : (
                <Download className="h-3.5 w-3.5 text-muted-foreground shrink-0" />
              )}
              <span className={`text-xs font-medium truncate ${isFailed ? "text-destructive" : ""}`}>
                {label}
              </span>
            </div>

            {!isFailed && (
              <Progress
                value={isExtracting || isCompleted ? 100 : dl.percent}
                className={`h-1.5 flex-1 ${isExtracting ? "animate-pulse" : ""}`}
              />
            )}

            <span className={`text-[11px] min-w-[100px] text-right ${isFailed ? "text-destructive" : "text-muted-foreground"}`}>
              {isFailed
                ? (dl.message || "Download failed")
                : isCompleted
                  ? (dl.message || "Completed")
                  : isExtracting
                    ? (dl.message || "Installing...")
                    : dl.total
                      ? `${formatBytes(dl.downloaded)} / ${formatBytes(dl.total)}`
                      : `${formatBytes(dl.downloaded)}`}
            </span>

            {!isFailed && !isCompleted && !isExtracting && (
              <span className="text-[11px] font-medium min-w-[36px] text-right">
                {`${dl.percent.toFixed(0)}%`}
              </span>
            )}
          </div>
        )
      })}
    </div>
  )
}
