export interface CertificateInfo {
  domain: string
  certPath: string
  keyPath: string
  exists: boolean
}

export interface DnsEntry {
  domain: string
  ip: string
}

export interface LogFile {
  name: string
  path: string
  size: number
}

export interface LogLine {
  file: string
  line: string
}
