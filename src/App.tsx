import { Routes, Route, Navigate } from "react-router"
import { MainLayout } from "@/components/layout/MainLayout"
import { Toaster } from "@/components/ui/sonner"
import { DashboardPage } from "@/pages/DashboardPage"
import { SitesPage } from "@/pages/SitesPage"
import { ServicesPage } from "@/pages/ServicesPage"
import { PhpPage } from "@/pages/PhpPage"
import { DatabasePage } from "@/pages/DatabasePage"
import { SslPage } from "@/pages/SslPage"
import { LogsPage } from "@/pages/LogsPage"
import { SettingsPage } from "@/pages/SettingsPage"
import { AiPage } from "@/pages/AiPage"

function App() {
  return (
    <>
      <Routes>
        <Route element={<MainLayout />}>
          <Route index element={<DashboardPage />} />
          <Route path="sites" element={<SitesPage />} />
          <Route path="services" element={<ServicesPage />} />
          <Route path="php" element={<PhpPage />} />
          <Route path="database" element={<DatabasePage />} />
          <Route path="ssl" element={<SslPage />} />
          <Route path="logs" element={<LogsPage />} />
          <Route path="settings" element={<SettingsPage />} />
          <Route path="ai" element={<AiPage />} />
          <Route path="*" element={<Navigate to="/" replace />} />
        </Route>
      </Routes>
      <Toaster position="top-right" richColors />
    </>
  )
}

export default App
