export interface NginxInfo {
  installed: boolean
  running: boolean
  version: string | null
  pid: number | null
  port: number
  configPath: string
}

export interface Site {
  id: string
  name: string
  domain: string
  documentRoot: string
  phpVersion: string
  ssl: boolean
  active: boolean
  createdAt: string
}
