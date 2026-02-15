export interface AppConfig {
  autoStartServices: boolean
  autoStartList: string[]
  defaultPhpVersion: string
  sitesDirectory: string
  tld: string
  nginxPort: number
  mariadbPort: number
  phpFpmBasePort: number
}

export interface SystemInfo {
  os: string
  arch: string
  dataDir: string
  appVersion: string
}
