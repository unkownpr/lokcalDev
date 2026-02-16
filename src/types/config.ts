export interface AppConfig {
  autoStartServices: boolean
  autoStartList: string[]
  defaultPhpVersion: string
  sitesDirectory: string
  tld: string
  nginxPort: number
  nginxSslPort: number
  mariadbPort: number
  phpFpmBasePort: number
  openrouterApiKey: string
  aiModel: string
  aiSystemPrompt: string
}

export interface SystemInfo {
  os: string
  arch: string
  dataDir: string
  appVersion: string
}
