import { useEffect, useRef, useState } from "react"
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
import { Textarea } from "@/components/ui/textarea"
import { ScrollArea } from "@/components/ui/scroll-area"
import { RefreshCw, Loader2, CheckCircle2, ExternalLink, Search, ChevronsUpDown, Check } from "lucide-react"
import { useSettingsStore } from "@/stores/settingsStore"
import { useAppStore } from "@/stores/appStore"
import { aiFetchModels } from "@/lib/tauri"
import type { AppConfig } from "@/types/config"
import type { AiModel } from "@/types/ai"

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
  const [showApiKey, setShowApiKey] = useState(false)
  const [models, setModels] = useState<AiModel[]>([])
  const [modelsLoading, setModelsLoading] = useState(false)
  const [modelsError, setModelsError] = useState<string | null>(null)
  const [modelSearch, setModelSearch] = useState("")
  const [modelDropdownOpen, setModelDropdownOpen] = useState(false)
  const modelDropdownRef = useRef<HTMLDivElement>(null)

  useEffect(() => {
    fetchSettings()
  }, [fetchSettings])

  useEffect(() => {
    if (config) {
      setForm({ ...config })
    }
  }, [config])

  // Fetch models when API key is available
  const fetchModels = async (apiKey: string) => {
    if (!apiKey || apiKey.length < 10) return
    setModelsLoading(true)
    setModelsError(null)
    try {
      const result = await aiFetchModels(apiKey)
      setModels(result)
    } catch (err) {
      setModelsError(String(err))
    } finally {
      setModelsLoading(false)
    }
  }

  useEffect(() => {
    if (form?.openrouterApiKey) {
      fetchModels(form.openrouterApiKey)
    }
  }, [form?.openrouterApiKey])

  // Close dropdown on click outside
  useEffect(() => {
    const handleClickOutside = (e: MouseEvent) => {
      if (modelDropdownRef.current && !modelDropdownRef.current.contains(e.target as Node)) {
        setModelDropdownOpen(false)
      }
    }
    document.addEventListener("mousedown", handleClickOutside)
    return () => document.removeEventListener("mousedown", handleClickOutside)
  }, [])

  const filteredModels = models.filter((m) => {
    const q = modelSearch.toLowerCase()
    return m.id.toLowerCase().includes(q) || m.name.toLowerCase().includes(q)
  })

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
          <TabsTrigger value="ai">AI</TabsTrigger>
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

        <TabsContent value="ai" className="mt-4 space-y-4">
          <Card className="p-4 space-y-4">
            <div className="space-y-1.5">
              <Label>OpenRouter API Key</Label>
              <div className="flex gap-2">
                <Input
                  type={showApiKey ? "text" : "password"}
                  value={form.openrouterApiKey}
                  onChange={(e) => setForm({ ...form, openrouterApiKey: e.target.value })}
                  placeholder="sk-or-..."
                />
                <Button
                  size="sm"
                  variant="outline"
                  onClick={() => setShowApiKey(!showApiKey)}
                  className="shrink-0"
                >
                  {showApiKey ? "Hide" : "Show"}
                </Button>
              </div>
              <p className="text-[11px] text-muted-foreground">
                Get your API key from{" "}
                <a
                  href="#"
                  onClick={(e) => {
                    e.preventDefault()
                    import("@tauri-apps/plugin-shell").then(({ open }) =>
                      open("https://openrouter.ai/keys"),
                    )
                  }}
                  className="text-primary underline"
                >
                  openrouter.ai/keys
                  <ExternalLink className="ml-0.5 inline h-3 w-3" />
                </a>
              </p>
            </div>
            <div className="space-y-1.5">
              <Label>AI Model</Label>
              <div className="relative" ref={modelDropdownRef}>
                <div
                  className="flex h-9 w-full cursor-pointer items-center justify-between rounded-md border border-input bg-background px-3 text-sm"
                  onClick={() => {
                    if (models.length > 0) setModelDropdownOpen(!modelDropdownOpen)
                  }}
                >
                  <span className={form.aiModel ? "text-foreground" : "text-muted-foreground"}>
                    {form.aiModel || "Select a model..."}
                  </span>
                  <div className="flex items-center gap-1.5">
                    {modelsLoading && <Loader2 className="h-3.5 w-3.5 animate-spin text-muted-foreground" />}
                    <ChevronsUpDown className="h-3.5 w-3.5 text-muted-foreground" />
                  </div>
                </div>

                {modelDropdownOpen && (
                  <div className="absolute z-50 mt-1 w-full rounded-md border border-border bg-popover shadow-md">
                    <div className="flex items-center border-b border-border px-3">
                      <Search className="mr-2 h-3.5 w-3.5 shrink-0 text-muted-foreground" />
                      <input
                        autoFocus
                        value={modelSearch}
                        onChange={(e) => setModelSearch(e.target.value)}
                        placeholder="Search models..."
                        className="h-9 w-full bg-transparent text-sm outline-none placeholder:text-muted-foreground"
                      />
                    </div>
                    <ScrollArea className="max-h-[250px] overflow-y-auto">
                      {filteredModels.length === 0 ? (
                        <p className="px-3 py-4 text-center text-xs text-muted-foreground">
                          {modelsLoading ? "Loading models..." : "No models found"}
                        </p>
                      ) : (
                        <div className="p-1">
                          {filteredModels.map((m) => (
                            <button
                              key={m.id}
                              onClick={() => {
                                setForm({ ...form, aiModel: m.id })
                                setModelDropdownOpen(false)
                                setModelSearch("")
                              }}
                              className="flex w-full items-center gap-2 rounded-sm px-2 py-1.5 text-left text-sm hover:bg-accent"
                            >
                              <Check
                                className={`h-3.5 w-3.5 shrink-0 ${form.aiModel === m.id ? "opacity-100" : "opacity-0"}`}
                              />
                              <div className="min-w-0 flex-1">
                                <div className="truncate text-xs font-medium">{m.name}</div>
                                <div className="truncate text-[10px] text-muted-foreground">{m.id}</div>
                              </div>
                            </button>
                          ))}
                        </div>
                      )}
                    </ScrollArea>
                  </div>
                )}
              </div>
              {modelsError && (
                <p className="text-[11px] text-destructive">{modelsError}</p>
              )}
              <p className="text-[11px] text-muted-foreground">
                {models.length > 0
                  ? `${models.length} models available — click to search and select`
                  : "Enter a valid API key to load models"}
              </p>
            </div>
            <div className="space-y-1.5">
              <Label>System Prompt</Label>
              <Textarea
                value={form.aiSystemPrompt}
                onChange={(e) => setForm({ ...form, aiSystemPrompt: e.target.value })}
                rows={4}
                placeholder="You are LokcalDev AI Assistant..."
              />
              <p className="text-[11px] text-muted-foreground">
                Customize the AI assistant's behavior and personality
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
