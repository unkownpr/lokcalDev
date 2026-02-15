export interface MariaDbInfo {
  installed: boolean
  initialized: boolean
  running: boolean
  version: string | null
  pid: number | null
  port: number
  dataDir: string
}

export interface DatabaseEntry {
  name: string
}

export interface PhpMyAdminInfo {
  installed: boolean
  version: string | null
  path: string
}
