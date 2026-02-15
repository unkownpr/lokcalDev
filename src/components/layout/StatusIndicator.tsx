import type { ServiceStatus } from "@/types/service"
import { cn } from "@/lib/utils"

const statusColors: Record<ServiceStatus, string> = {
  running: "bg-emerald-500",
  stopped: "bg-neutral-300",
  error: "bg-red-500",
  starting: "bg-amber-400 animate-pulse",
  stopping: "bg-amber-400 animate-pulse",
}

interface StatusIndicatorProps {
  status: ServiceStatus
  size?: "sm" | "md"
}

export function StatusIndicator({ status, size = "sm" }: StatusIndicatorProps) {
  return (
    <span
      className={cn(
        "inline-block rounded-full",
        statusColors[status],
        size === "sm" ? "h-2 w-2" : "h-2.5 w-2.5",
      )}
    />
  )
}
