import { useEffect, useState, useMemo, useRef, useCallback } from "react"
import { Play, Square, RotateCw, Trash2, Search, Pencil, Check, X, Plus } from "lucide-react"
import { Card } from "@/components/ui/card"
import { Button } from "@/components/ui/button"
import { Badge } from "@/components/ui/badge"
import { Switch } from "@/components/ui/switch"
import { Input } from "@/components/ui/input"
import { Label } from "@/components/ui/label"
import { Separator } from "@/components/ui/separator"
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from "@/components/ui/select"
import { Tabs, TabsContent, TabsList, TabsTrigger } from "@/components/ui/tabs"
import { PageHeader } from "@/components/shared/PageHeader"
import { DownloadButton } from "@/components/shared/DownloadButton"
import { StatusIndicator } from "@/components/layout/StatusIndicator"
import { usePhpStore } from "@/stores/phpStore"
import { useDownloadProgress } from "@/hooks/useDownloadProgress"

const QUICK_SETTINGS = [
  { key: "memory_limit", label: "Memory Limit", desc: "Maximum memory a script can use", type: "size" as const },
  { key: "upload_max_filesize", label: "Upload Max Filesize", desc: "Maximum upload file size", type: "size" as const },
  { key: "post_max_size", label: "Post Max Size", desc: "Maximum POST data size", type: "size" as const },
  { key: "max_execution_time", label: "Max Execution Time", desc: "Script timeout in seconds", type: "number" as const, suffix: "sec" },
  { key: "max_input_time", label: "Max Input Time", desc: "Input parsing timeout", type: "number" as const, suffix: "sec" },
  { key: "max_file_uploads", label: "Max File Uploads", desc: "Simultaneous file uploads", type: "number" as const },
  { key: "display_errors", label: "Display Errors", desc: "Show errors in browser", type: "toggle" as const },
  { key: "display_startup_errors", label: "Startup Errors", desc: "Show startup errors", type: "toggle" as const },
  { key: "error_reporting", label: "Error Reporting", desc: "Error reporting level", type: "select" as const,
    options: ["E_ALL", "E_ALL & ~E_NOTICE", "E_ALL & ~E_NOTICE & ~E_DEPRECATED", "E_ERROR | E_PARSE"] },
  { key: "date.timezone", label: "Timezone", desc: "Default timezone", type: "select" as const,
    options: ["UTC", "Europe/Istanbul", "Europe/London", "America/New_York", "America/Los_Angeles", "Asia/Tokyo", "Asia/Shanghai", "Asia/Dubai", "Australia/Sydney", "Pacific/Auckland"] },
]

export function PhpPage() {
  const {
    versions,
    extensions,
    iniDirectives,
    installing,
    fetchVersions,
    installVersion,
    removeVersion,
    startFpm,
    stopFpm,
    restartFpm,
    fetchExtensions,
    toggleExtension,
    fetchIni,
    setIniDirective,
  } = usePhpStore()

  const progress = useDownloadProgress()
  const [selectedVersion, setSelectedVersion] = useState("8.3")
  const [iniKey, setIniKey] = useState("")
  const [iniValue, setIniValue] = useState("")
  const [searchQuery, setSearchQuery] = useState("")
  const [editingKey, setEditingKey] = useState<string | null>(null)
  const [editValue, setEditValue] = useState("")

  // Debounced ini directive setter for Quick Settings
  const debounceTimers = useRef<Record<string, ReturnType<typeof setTimeout>>>({})
  const debouncedSetIni = useCallback(
    (version: string, key: string, value: string) => {
      if (debounceTimers.current[key]) {
        clearTimeout(debounceTimers.current[key])
      }
      debounceTimers.current[key] = setTimeout(() => {
        setIniDirective(version, key, value)
        delete debounceTimers.current[key]
      }, 500)
    },
    [setIniDirective],
  )

  useEffect(() => {
    fetchVersions()
  }, [fetchVersions])

  const installedVersions = versions.filter((v) => v.installed)

  useEffect(() => {
    if (installedVersions.length > 0 && installedVersions.find((v) => v.version === selectedVersion)) {
      fetchExtensions(selectedVersion)
      fetchIni(selectedVersion)
    }
  }, [selectedVersion, versions, fetchExtensions, fetchIni])

  const filteredDirectives = useMemo(() => {
    if (!searchQuery.trim()) return iniDirectives
    const q = searchQuery.toLowerCase()
    return iniDirectives.filter(
      (d) => d.key.toLowerCase().includes(q) || d.value.toLowerCase().includes(q)
    )
  }, [iniDirectives, searchQuery])

  return (
    <div>
      <PageHeader title="PHP" description="Manage PHP versions and extensions" />

      <Tabs defaultValue="versions">
        <TabsList>
          <TabsTrigger value="versions">Versions</TabsTrigger>
          <TabsTrigger value="extensions">Extensions</TabsTrigger>
          <TabsTrigger value="config">Configuration</TabsTrigger>
        </TabsList>

        <TabsContent value="versions" className="mt-4">
          <div className="grid grid-cols-2 gap-4">
            {versions.map((v) => (
              <Card key={v.version} className="p-4">
                <div className="flex items-center justify-between mb-3">
                  <div className="flex items-center gap-2">
                    <StatusIndicator status={v.running ? "running" : v.installed ? "stopped" : "stopped"} />
                    <span className="font-medium text-sm">PHP {v.version}</span>
                  </div>
                  <Badge variant={v.installed ? (v.running ? "default" : "secondary") : "outline"}>
                    {v.installed ? (v.running ? "Running" : "Installed") : "Not Installed"}
                  </Badge>
                </div>

                {v.installed && (
                  <div className="text-xs text-muted-foreground mb-3 space-y-0.5">
                    <p>Port: {v.port}</p>
                    {v.pid && <p>PID: {v.pid}</p>}
                  </div>
                )}

                <div className="flex gap-2">
                  {!v.installed ? (
                    <DownloadButton
                      label="Install"
                      progress={progress[`php-${v.version}`]}
                      onClick={() => installVersion(v.version)}
                      disabled={installing !== null}
                    />
                  ) : (
                    <>
                      {!v.running ? (
                        <Button size="sm" variant="outline" onClick={() => startFpm(v.version)}>
                          <Play className="mr-1 h-3 w-3" /> Start
                        </Button>
                      ) : (
                        <Button size="sm" variant="outline" onClick={() => stopFpm(v.version)}>
                          <Square className="mr-1 h-3 w-3" /> Stop
                        </Button>
                      )}
                      {v.running && (
                        <Button size="sm" variant="outline" onClick={() => restartFpm(v.version)}>
                          <RotateCw className="mr-1 h-3 w-3" /> Restart
                        </Button>
                      )}
                      {!v.running && (
                        <Button size="sm" variant="destructive" onClick={() => removeVersion(v.version)}>
                          <Trash2 className="mr-1 h-3 w-3" /> Remove
                        </Button>
                      )}
                    </>
                  )}
                </div>
              </Card>
            ))}
          </div>
        </TabsContent>

        <TabsContent value="extensions" className="mt-4">
          {installedVersions.length === 0 ? (
            <p className="text-sm text-muted-foreground">Install a PHP version first.</p>
          ) : (
            <>
              <div className="mb-4 flex gap-2">
                {installedVersions.map((v) => (
                  <Button
                    key={v.version}
                    size="sm"
                    variant={selectedVersion === v.version ? "default" : "outline"}
                    onClick={() => setSelectedVersion(v.version)}
                  >
                    PHP {v.version}
                  </Button>
                ))}
              </div>
              <div className="space-y-1">
                {extensions.map((ext) => (
                  <div key={ext.name} className="flex items-center justify-between py-1.5 px-2 rounded hover:bg-muted/50">
                    <span className="text-sm">{ext.name}</span>
                    <div className="flex items-center gap-2">
                      {ext.builtin && (
                        <Badge variant="outline" className="text-[10px]">built-in</Badge>
                      )}
                      <Switch
                        checked={ext.enabled}
                        onCheckedChange={(checked) => toggleExtension(selectedVersion, ext.name, checked)}
                        disabled={ext.builtin}
                      />
                    </div>
                  </div>
                ))}
              </div>
            </>
          )}
        </TabsContent>

        <TabsContent value="config" className="mt-4">
          {installedVersions.length === 0 ? (
            <p className="text-sm text-muted-foreground">Install a PHP version first.</p>
          ) : (
            <>
              <div className="mb-4 flex gap-2">
                {installedVersions.map((v) => (
                  <Button
                    key={v.version}
                    size="sm"
                    variant={selectedVersion === v.version ? "default" : "outline"}
                    onClick={() => setSelectedVersion(v.version)}
                  >
                    PHP {v.version}
                  </Button>
                ))}
              </div>

              {/* Quick Settings */}
              <Card className="p-4 mb-6">
                <h3 className="text-sm font-medium mb-4">Quick Settings</h3>
                <div className="grid grid-cols-2 gap-x-8 gap-y-4">
                  {QUICK_SETTINGS.map((setting) => {
                    const directive = iniDirectives.find((d) => d.key === setting.key)
                    const currentValue = directive?.value ?? ""

                    if (setting.type === "size") {
                      const numericPart = currentValue.replace(/[^\d]/g, "")
                      const suffixPart = currentValue.replace(/[\d]/g, "").trim() || "M"
                      return (
                        <div key={setting.key} className="space-y-1.5">
                          <Label className="text-sm">{setting.label}</Label>
                          <p className="text-xs text-muted-foreground">{setting.desc}</p>
                          <div className="flex gap-1.5">
                            <Input
                              type="number"
                              value={numericPart}
                              className="flex-1"
                              onChange={(e) => {
                                const newVal = e.target.value + suffixPart
                                debouncedSetIni(selectedVersion, setting.key, newVal)
                              }}
                            />
                            <Select
                              value={suffixPart}
                              onValueChange={(s) => {
                                const newVal = numericPart + s
                                debouncedSetIni(selectedVersion, setting.key, newVal)
                              }}
                            >
                              <SelectTrigger className="w-18">
                                <SelectValue />
                              </SelectTrigger>
                              <SelectContent>
                                <SelectItem value="K">K</SelectItem>
                                <SelectItem value="M">M</SelectItem>
                                <SelectItem value="G">G</SelectItem>
                              </SelectContent>
                            </Select>
                          </div>
                        </div>
                      )
                    }

                    if (setting.type === "number") {
                      return (
                        <div key={setting.key} className="space-y-1.5">
                          <Label className="text-sm">{setting.label}</Label>
                          <p className="text-xs text-muted-foreground">{setting.desc}</p>
                          <div className="flex gap-1.5 items-center">
                            <Input
                              type="number"
                              value={currentValue}
                              className="flex-1"
                              onChange={(e) => {
                                debouncedSetIni(selectedVersion, setting.key, e.target.value)
                              }}
                            />
                            {setting.suffix && (
                              <span className="text-xs text-muted-foreground w-8">{setting.suffix}</span>
                            )}
                          </div>
                        </div>
                      )
                    }

                    if (setting.type === "toggle") {
                      const isOn = currentValue === "On" || currentValue === "1" || currentValue === "true"
                      return (
                        <div key={setting.key} className="flex items-center justify-between">
                          <div className="space-y-0.5">
                            <Label className="text-sm">{setting.label}</Label>
                            <p className="text-xs text-muted-foreground">{setting.desc}</p>
                          </div>
                          <Switch
                            checked={isOn}
                            onCheckedChange={(checked) => {
                              debouncedSetIni(selectedVersion, setting.key, checked ? "On" : "Off")
                            }}
                          />
                        </div>
                      )
                    }

                    if (setting.type === "select") {
                      return (
                        <div key={setting.key} className="space-y-1.5">
                          <Label className="text-sm">{setting.label}</Label>
                          <p className="text-xs text-muted-foreground">{setting.desc}</p>
                          <Select
                            value={currentValue}
                            onValueChange={(val) => {
                              debouncedSetIni(selectedVersion, setting.key, val)
                            }}
                          >
                            <SelectTrigger className="w-full">
                              <SelectValue placeholder="Select..." />
                            </SelectTrigger>
                            <SelectContent>
                              {setting.options.map((opt) => (
                                <SelectItem key={opt} value={opt}>
                                  {opt}
                                </SelectItem>
                              ))}
                            </SelectContent>
                          </Select>
                        </div>
                      )
                    }

                    return null
                  })}
                </div>
              </Card>

              {/* All Directives */}
              <div>
                <div className="flex items-center justify-between mb-3">
                  <h3 className="text-sm font-medium">All Directives</h3>
                  <div className="relative w-64">
                    <Search className="absolute left-2.5 top-2.5 h-3.5 w-3.5 text-muted-foreground" />
                    <Input
                      placeholder="Search directives..."
                      value={searchQuery}
                      onChange={(e) => setSearchQuery(e.target.value)}
                      className="pl-8 h-9"
                    />
                  </div>
                </div>

                <div className="space-y-0.5 mb-4">
                  {filteredDirectives.map((d, i) => (
                    <div
                      key={`${d.key}-${i}`}
                      className="flex items-center justify-between py-1.5 px-2 rounded hover:bg-muted/50 group"
                    >
                      <div className="flex items-center gap-2 min-w-0 flex-1">
                        <Badge variant="outline" className="text-[10px] font-normal shrink-0">
                          {d.section}
                        </Badge>
                        <span className="text-sm font-mono truncate">{d.key}</span>
                      </div>
                      {editingKey === d.key ? (
                        <div className="flex items-center gap-1.5 ml-4">
                          <Input
                            value={editValue}
                            onChange={(e) => setEditValue(e.target.value)}
                            className="h-7 w-48 text-sm font-mono"
                            autoFocus
                            onKeyDown={(e) => {
                              if (e.key === "Enter") {
                                setIniDirective(selectedVersion, d.key, editValue)
                                setEditingKey(null)
                              }
                              if (e.key === "Escape") {
                                setEditingKey(null)
                              }
                            }}
                          />
                          <Button
                            size="icon"
                            variant="ghost"
                            className="h-7 w-7"
                            onClick={() => {
                              setIniDirective(selectedVersion, d.key, editValue)
                              setEditingKey(null)
                            }}
                          >
                            <Check className="h-3.5 w-3.5" />
                          </Button>
                          <Button
                            size="icon"
                            variant="ghost"
                            className="h-7 w-7"
                            onClick={() => setEditingKey(null)}
                          >
                            <X className="h-3.5 w-3.5" />
                          </Button>
                        </div>
                      ) : (
                        <div className="flex items-center gap-1.5 ml-4">
                          <span className="text-sm font-mono text-muted-foreground truncate max-w-48">
                            {d.value}
                          </span>
                          <Button
                            size="icon"
                            variant="ghost"
                            className="h-7 w-7 opacity-0 group-hover:opacity-100 transition-opacity"
                            onClick={() => {
                              setEditingKey(d.key)
                              setEditValue(d.value)
                            }}
                          >
                            <Pencil className="h-3 w-3" />
                          </Button>
                        </div>
                      )}
                    </div>
                  ))}
                  {filteredDirectives.length === 0 && searchQuery && (
                    <p className="text-sm text-muted-foreground py-4 text-center">
                      No directives matching "{searchQuery}"
                    </p>
                  )}
                </div>

                <Separator className="my-4" />

                {/* Add New Directive */}
                <Card className="p-3">
                  <div className="flex items-center gap-2">
                    <Plus className="h-4 w-4 text-muted-foreground shrink-0" />
                    <Input
                      placeholder="Directive key"
                      value={iniKey}
                      onChange={(e) => setIniKey(e.target.value)}
                      className="flex-1 h-8"
                    />
                    <Input
                      placeholder="Value"
                      value={iniValue}
                      onChange={(e) => setIniValue(e.target.value)}
                      className="flex-1 h-8"
                    />
                    <Button
                      size="sm"
                      onClick={() => {
                        if (iniKey && iniValue) {
                          setIniDirective(selectedVersion, iniKey, iniValue)
                          setIniKey("")
                          setIniValue("")
                        }
                      }}
                      disabled={!iniKey || !iniValue}
                    >
                      Set
                    </Button>
                  </div>
                </Card>
              </div>
            </>
          )}
        </TabsContent>
      </Tabs>
    </div>
  )
}
