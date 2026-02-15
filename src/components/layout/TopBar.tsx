import { StatusIndicator } from "./StatusIndicator"
import { useServiceStore } from "@/stores/serviceStore"

export function TopBar() {
  const services = useServiceStore((s) => s.services)

  return (
    <header className="flex h-12 items-center justify-between border-b border-border bg-background px-6">
      <div />
      <div className="flex items-center gap-4">
        {services.map((service) => (
          <div key={service.id} className="flex items-center gap-1.5">
            <StatusIndicator status={service.status} />
            <span className="text-[11px] text-muted-foreground">
              {service.name}
            </span>
          </div>
        ))}
      </div>
    </header>
  )
}
