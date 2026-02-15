import { PageHeader } from "@/components/shared/PageHeader"
import { ServiceCard } from "@/components/shared/ServiceCard"
import { useServiceStore } from "@/stores/serviceStore"

export function ServicesPage() {
  const services = useServiceStore((s) => s.services)
  const installService = useServiceStore((s) => s.installService)
  const initializeService = useServiceStore((s) => s.initializeService)
  const startService = useServiceStore((s) => s.startService)
  const stopService = useServiceStore((s) => s.stopService)
  const restartService = useServiceStore((s) => s.restartService)

  return (
    <div>
      <PageHeader
        title="Services"
        description="Manage your local development services"
      />

      <div className="grid grid-cols-2 gap-3">
        {services.map((service) => (
          <ServiceCard
            key={service.id}
            service={service}
            onInstall={installService}
            onInitialize={initializeService}
            onStart={startService}
            onStop={stopService}
            onRestart={restartService}
          />
        ))}
      </div>
    </div>
  )
}
