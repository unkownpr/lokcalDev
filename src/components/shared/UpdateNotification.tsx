import { Button } from "@/components/ui/button"
import { ArrowDownToLine, Loader2, RefreshCw } from "lucide-react"
import { useAppStore } from "@/stores/appStore"

export function UpdateNotification() {
  const { updateAvailable, updateVersion, updateInstalling, updateProgress, installUpdate } =
    useAppStore()

  if (!updateAvailable) return null

  const progressPercent =
    updateProgress?.total && updateProgress.total > 0
      ? Math.round((updateProgress.downloaded / updateProgress.total) * 100)
      : null

  return (
    <div className="mx-3 mb-2 rounded-md border border-primary/20 bg-primary/5 p-2.5">
      <div className="flex items-center gap-2 text-xs font-medium text-primary">
        <ArrowDownToLine className="h-3.5 w-3.5" />
        v{updateVersion} available
      </div>
      {updateInstalling ? (
        <div className="mt-1.5 flex items-center gap-2">
          <Loader2 className="h-3 w-3 animate-spin text-muted-foreground" />
          <span className="text-[11px] text-muted-foreground">
            {progressPercent !== null ? `Downloading ${progressPercent}%` : "Installing..."}
          </span>
        </div>
      ) : (
        <Button
          size="sm"
          className="mt-1.5 h-6 w-full text-[11px]"
          onClick={installUpdate}
        >
          <RefreshCw className="mr-1 h-3 w-3" />
          Update & Restart
        </Button>
      )}
    </div>
  )
}
