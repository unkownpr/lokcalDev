import { NavLink } from "react-router"
import {
  LayoutDashboard,
  Globe,
  Server,
  FileCode,
  Database,
  Shield,
  ScrollText,
  Settings,
  Bot,
  Github,
  ExternalLink,
} from "lucide-react"
import { cn } from "@/lib/utils"
import { APP_NAME } from "@/lib/constants"
import { Logo } from "@/components/shared/Logo"
import { UpdateNotification } from "@/components/shared/UpdateNotification"
import { useAppStore } from "@/stores/appStore"
import { open } from "@tauri-apps/plugin-shell"

const iconMap = {
  LayoutDashboard,
  Globe,
  Server,
  FileCode,
  Database,
  Shield,
  ScrollText,
  Settings,
  Bot,
} as const

const navItems = [
  { path: "/", label: "Dashboard", icon: "LayoutDashboard" as const },
  { path: "/sites", label: "Sites", icon: "Globe" as const },
  { path: "/services", label: "Services", icon: "Server" as const },
  { path: "/php", label: "PHP", icon: "FileCode" as const },
  { path: "/database", label: "Database", icon: "Database" as const },
  { path: "/ssl", label: "SSL", icon: "Shield" as const },
  { path: "/logs", label: "Logs", icon: "ScrollText" as const },
  { path: "/ai", label: "AI Assistant", icon: "Bot" as const },
  { path: "/settings", label: "Settings", icon: "Settings" as const },
]

export function Sidebar() {
  const appVersion = useAppStore((s) => s.systemInfo?.appVersion)
  const updateAvailable = useAppStore((s) => s.updateAvailable)
  return (
    <aside className="flex h-screen w-56 flex-col border-r border-border bg-sidebar">
      <div className="flex h-14 items-center gap-2.5 px-4">
        <Logo size={26} />
        <h1 className="text-sm font-semibold tracking-tight text-foreground">
          {APP_NAME}
        </h1>
      </div>

      <nav className="flex-1 space-y-0.5 px-3 py-2">
        {navItems.map((item) => {
          const Icon = iconMap[item.icon]
          return (
            <NavLink
              key={item.path}
              to={item.path}
              end={item.path === "/"}
              className={({ isActive }) =>
                cn(
                  "flex items-center gap-3 rounded-md px-2.5 py-1.5 text-[13px] font-medium transition-colors",
                  isActive
                    ? "bg-sidebar-accent text-sidebar-accent-foreground"
                    : "text-muted-foreground hover:bg-sidebar-accent/50 hover:text-sidebar-foreground",
                )
              }
            >
              <Icon className="h-4 w-4" />
              {item.label}
            </NavLink>
          )
        })}
      </nav>

      <UpdateNotification />
      <div className="border-t border-border px-3 py-3 space-y-2">
        <button
          onClick={() => open("https://github.com/unkownpr/lokcalDev")}
          className="flex w-full items-center gap-2 rounded-md px-2.5 py-1.5 text-[12px] font-medium text-muted-foreground transition-colors hover:bg-sidebar-accent/50 hover:text-sidebar-foreground"
        >
          <Github className="h-3.5 w-3.5" />
          Source Code
          <ExternalLink className="ml-auto h-3 w-3 opacity-50" />
        </button>
        <div className="flex items-center justify-between px-2.5">
          <button
            onClick={() => open("https://ssilistre.dev")}
            className="flex items-center gap-1.5 transition-opacity hover:opacity-80"
          >
            <img
              src="https://ssilistre.dev/public/images/ssilistre_face.png"
              alt="ssilistre.dev"
              className="h-5 w-5 rounded-full"
            />
            <span className="text-[10px] text-muted-foreground">ssilistre.dev</span>
          </button>
          <span className="flex items-center gap-1.5 text-[10px] text-muted-foreground">
            v{appVersion ?? "0.1.0"}
            {updateAvailable && (
              <span className="h-1.5 w-1.5 rounded-full bg-primary animate-pulse" />
            )}
          </span>
        </div>
      </div>
    </aside>
  )
}
