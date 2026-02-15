export interface PhpVersion {
  version: string
  installed: boolean
  running: boolean
  port: number
  pid: number | null
  path: string | null
}

export interface PhpIniDirective {
  key: string
  value: string
  section: string
}

export interface PhpExtension {
  name: string
  enabled: boolean
  builtin: boolean
}

export interface DownloadProgress {
  id: string
  downloaded: number
  total: number | null
  percent: number
  status: string
  message: string | null
}
