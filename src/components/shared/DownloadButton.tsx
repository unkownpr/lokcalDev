import { Button } from "@/components/ui/button"
import { Progress } from "@/components/ui/progress"
import { Download, Loader2, XCircle } from "lucide-react"
import type { DownloadProgress } from "@/types/php"

interface DownloadButtonProps {
  label: string
  progress?: DownloadProgress
  onClick: () => void
  disabled?: boolean
}

export function DownloadButton({ label, progress, onClick, disabled }: DownloadButtonProps) {
  const status = progress?.status

  if (status === "downloading" || status === "extracting") {
    return (
      <div className="space-y-1.5 w-full">
        <div className="flex items-center gap-2">
          <Loader2 className="h-3 w-3 animate-spin text-muted-foreground" />
          <span className="text-xs text-muted-foreground">
            {status === "extracting"
              ? "Extracting..."
              : `${progress!.percent.toFixed(0)}%`}
          </span>
        </div>
        <Progress
          value={status === "extracting" ? 100 : progress!.percent}
          className={`h-1.5 ${status === "extracting" ? "animate-pulse" : ""}`}
        />
      </div>
    )
  }

  if (status === "failed") {
    return (
      <div className="flex items-center gap-2">
        <XCircle className="h-3.5 w-3.5 text-destructive" />
        <span className="text-xs text-destructive">Failed</span>
        <Button size="sm" variant="outline" onClick={onClick} disabled={disabled}>
          Retry
        </Button>
      </div>
    )
  }

  return (
    <Button size="sm" variant="outline" onClick={onClick} disabled={disabled}>
      <Download className="mr-1.5 h-3.5 w-3.5" />
      {label}
    </Button>
  )
}
