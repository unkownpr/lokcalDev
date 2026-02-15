import { Outlet } from "react-router"
import { useEffect } from "react"
import { Sidebar } from "./Sidebar"
import { TopBar } from "./TopBar"
import { GlobalDownloadBar } from "./GlobalDownloadBar"
import { useAppStore } from "@/stores/appStore"
import { useServiceStore } from "@/stores/serviceStore"

export function MainLayout() {
  const initialize = useAppStore((s) => s.initialize)
  const fetchServices = useServiceStore((s) => s.fetchServices)

  useEffect(() => {
    initialize()
    fetchServices()
  }, [initialize, fetchServices])

  // Periodically refresh services to stay in sync with individual pages
  useEffect(() => {
    const interval = setInterval(fetchServices, 5000)
    return () => clearInterval(interval)
  }, [fetchServices])

  return (
    <div className="flex h-screen overflow-hidden bg-background">
      <Sidebar />
      <div className="flex flex-1 flex-col overflow-hidden">
        <TopBar />
        <main className="flex-1 overflow-y-auto p-6">
          <Outlet />
        </main>
        <GlobalDownloadBar />
      </div>
    </div>
  )
}
