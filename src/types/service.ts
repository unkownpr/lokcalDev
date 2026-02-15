export type ServiceStatus = "running" | "stopped" | "error" | "starting" | "stopping"

export interface ServiceInfo {
  id: string
  name: string
  status: ServiceStatus
  port: number | null
  version: string | null
  pid: number | null
  installed: boolean
  initialized: boolean
}
