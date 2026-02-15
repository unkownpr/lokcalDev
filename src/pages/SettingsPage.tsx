import { useEffect, useState } from "react"
import { Card } from "@/components/ui/card"
import { Button } from "@/components/ui/button"
import { Input } from "@/components/ui/input"
import { Label } from "@/components/ui/label"
import { Switch } from "@/components/ui/switch"
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from "@/components/ui/select"
import { Tabs, TabsContent, TabsList, TabsTrigger } from "@/components/ui/tabs"
import { PageHeader } from "@/components/shared/PageHeader"
import { RefreshCw, Loader2, CheckCircle2 } from "lucide-react"
import { useSettingsStore } from "@/stores/settingsStore"
import { useAppStore } from "@/stores/appStore"
import type { AppConfig } from "@/types/config"

export function SettingsPage() {
  const { config, fetchSettings, saveSettings, resetSettings } = useSettingsStore()
  const systemInfo = useAppStore((s) => s.systemInfo)
  const {
    updateAvailable,
    updateVersion,
    updateChecking,
    updateInstalling,
    checkForUpdate: checkUpdate,
    installUpdate,
  } = useAppStore()
  const [form, setForm] = useState<AppConfig | null>(null)
  const [saved, setSaved] = useState(false)

  useEffect(() => {
    fetchSettings()
  }, [fetchSettings])

  useEffect(() => {
    if (config) {
      setForm({ ...config })
    }
  }, [config])

  const handleSave = async () => {
    if (!form) return
    await saveSettings(form)
    setSaved(true)
    setTimeout(() => setSaved(false), 2000)
  }

  if (!form) return null

  return (
    <div>
      <PageHeader title="Settings" description="Configure LokcalDev preferences">
        <div className="flex gap-2">
          <Button size="sm" variant="outline" onClick={resetSettings}>
            Reset to Defaults
          </Button>
          <Button size="sm" onClick={handleSave}>
            {saved ? "Saved!" : "Save Settings"}
          </Button>
        </div>
      </PageHeader>

      <Tabs defaultValue="general">
        <TabsList>
          <TabsTrigger value="general">General</TabsTrigger>
          <TabsTrigger value="services">Services</TabsTrigger>
          <TabsTrigger value="ports">Ports</TabsTrigger>
          <TabsTrigger value="about">About</TabsTrigger>
        </TabsList>

        <TabsContent value="general" className="mt-4 space-y-4">
          <Card className="p-4 space-y-4">
            <div className="space-y-1.5">
              <Label>Sites Directory</Label>
              <Input
                value={form.sitesDirectory}
                onChange={(e) => setForm({ ...form, sitesDirectory: e.target.value })}
              />
              <p className="text-[11px] text-muted-foreground">Default document root for new sites</p>
            </div>
            <div className="space-y-1.5">
              <Label>Top-Level Domain</Label>
              <Input
                value={form.tld}
                onChange={(e) => setForm({ ...form, tld: e.target.value })}
              />
              <p className="text-[11px] text-muted-foreground">Domain suffix for local sites (e.g. .test)</p>
            </div>
          </Card>
        </TabsContent>

        <TabsContent value="services" className="mt-4 space-y-4">
          <Card className="p-4 space-y-4">
            <div className="flex items-center justify-between">
              <div>
                <Label>Auto-start Services</Label>
                <p className="text-[11px] text-muted-foreground">Start services when LokcalDev launches</p>
              </div>
              <Switch
                checked={form.autoStartServices}
                onCheckedChange={(checked) => setForm({ ...form, autoStartServices: checked })}
              />
            </div>
            <div className="space-y-1.5">
              <Label>Default PHP Version</Label>
              <Select
                value={form.defaultPhpVersion}
                onValueChange={(value) => setForm({ ...form, defaultPhpVersion: value })}
              >
                <SelectTrigger>
                  <SelectValue />
                </SelectTrigger>
                <SelectContent>
                  <SelectItem value="8.1">PHP 8.1</SelectItem>
                  <SelectItem value="8.2">PHP 8.2</SelectItem>
                  <SelectItem value="8.3">PHP 8.3</SelectItem>
                  <SelectItem value="8.4">PHP 8.4</SelectItem>
                </SelectContent>
              </Select>
            </div>
          </Card>
        </TabsContent>

        <TabsContent value="ports" className="mt-4 space-y-4">
          <Card className="p-4 space-y-4">
            <div className="space-y-1.5">
              <Label>Nginx Port</Label>
              <Input
                type="number"
                value={form.nginxPort}
                onChange={(e) => setForm({ ...form, nginxPort: parseInt(e.target.value) || 80 })}
              />
            </div>
            <div className="space-y-1.5">
              <Label>MariaDB Port</Label>
              <Input
                type="number"
                value={form.mariadbPort}
                onChange={(e) => setForm({ ...form, mariadbPort: parseInt(e.target.value) || 3306 })}
              />
            </div>
            <div className="space-y-1.5">
              <Label>PHP-FPM Base Port</Label>
              <Input
                type="number"
                value={form.phpFpmBasePort}
                onChange={(e) => setForm({ ...form, phpFpmBasePort: parseInt(e.target.value) || 9081 })}
              />
              <p className="text-[11px] text-muted-foreground">
                PHP 8.1 = {form.phpFpmBasePort}, 8.2 = {form.phpFpmBasePort + 1}, 8.3 = {form.phpFpmBasePort + 2}, 8.4 = {form.phpFpmBasePort + 3}
              </p>
            </div>
          </Card>
        </TabsContent>

        <TabsContent value="about" className="mt-4 space-y-4">
          <Card className="p-4 space-y-3">
            <div className="flex items-center justify-between">
              <span className="text-sm text-muted-foreground">Application</span>
              <span className="text-sm font-medium">LokcalDev</span>
            </div>
            <div className="flex items-center justify-between">
              <span className="text-sm text-muted-foreground">Version</span>
              <span className="text-sm font-medium">v{systemInfo?.appVersion ?? "0.1.0"}</span>
            </div>
            <div className="flex items-center justify-between">
              <span className="text-sm text-muted-foreground">Platform</span>
              <span className="text-sm font-medium">{systemInfo?.os ?? "-"} / {systemInfo?.arch ?? "-"}</span>
            </div>
            <div className="flex items-center justify-between">
              <span className="text-sm text-muted-foreground">Data Directory</span>
              <span className="text-sm font-mono text-xs truncate max-w-[300px]">{systemInfo?.dataDir ?? "-"}</span>
            </div>
          </Card>

          <Card className="p-4 space-y-3">
            <div className="flex items-center justify-between">
              <div>
                <Label>Software Updates</Label>
                <p className="text-[11px] text-muted-foreground">
                  {updateAvailable
                    ? `Version ${updateVersion} is available`
                    : "You're on the latest version"}
                </p>
              </div>
              <div className="flex items-center gap-2">
                {updateAvailable ? (
                  <Button
                    size="sm"
                    onClick={installUpdate}
                    disabled={updateInstalling}
                  >
                    {updateInstalling ? (
                      <Loader2 className="mr-1.5 h-3.5 w-3.5 animate-spin" />
                    ) : (
                      <RefreshCw className="mr-1.5 h-3.5 w-3.5" />
                    )}
                    {updateInstalling ? "Installing..." : "Update Now"}
                  </Button>
                ) : (
                  <Button
                    size="sm"
                    variant="outline"
                    onClick={checkUpdate}
                    disabled={updateChecking}
                  >
                    {updateChecking ? (
                      <Loader2 className="mr-1.5 h-3.5 w-3.5 animate-spin" />
                    ) : (
                      <CheckCircle2 className="mr-1.5 h-3.5 w-3.5" />
                    )}
                    {updateChecking ? "Checking..." : "Check for Updates"}
                  </Button>
                )}
              </div>
            </div>
          </Card>
        </TabsContent>
      </Tabs>
    </div>
  )
}
