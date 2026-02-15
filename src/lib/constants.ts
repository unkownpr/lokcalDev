export const APP_NAME = "LokcalDev"

export const SERVICE_IDS = {
  NGINX: "nginx",
  PHP_FPM: "php-fpm",
  MARIADB: "mariadb",
  DNSMASQ: "dnsmasq",
} as const

export const NAV_ITEMS = [
  { path: "/", label: "Dashboard", icon: "LayoutDashboard" },
  { path: "/sites", label: "Sites", icon: "Globe" },
  { path: "/services", label: "Services", icon: "Server" },
  { path: "/php", label: "PHP", icon: "FileCode" },
  { path: "/database", label: "Database", icon: "Database" },
  { path: "/ssl", label: "SSL", icon: "Shield" },
  { path: "/logs", label: "Logs", icon: "ScrollText" },
  { path: "/settings", label: "Settings", icon: "Settings" },
] as const
