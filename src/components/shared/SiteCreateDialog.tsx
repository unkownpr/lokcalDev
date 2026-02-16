import { useState, useEffect } from "react"
import { Button } from "@/components/ui/button"
import {
  Dialog,
  DialogContent,
  DialogHeader,
  DialogTitle,
  DialogTrigger,
} from "@/components/ui/dialog"
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
import { open as openDialog } from "@tauri-apps/plugin-dialog"
import { Plus, FolderOpen } from "lucide-react"

interface SiteCreateDialogProps {
  onSubmit: (
    name: string,
    domain: string,
    documentRoot: string,
    phpVersion: string,
    ssl: boolean,
    template?: string,
  ) => Promise<void>
  phpVersions?: string[]
}

export function SiteCreateDialog({ onSubmit, phpVersions = [] }: SiteCreateDialogProps) {
  const [open, setOpen] = useState(false)
  const [name, setName] = useState("")
  const [domain, setDomain] = useState("")
  const [documentRoot, setDocumentRoot] = useState("")
  const [phpVersion, setPhpVersion] = useState(phpVersions[0] ?? "")
  const [ssl, setSsl] = useState(false)
  const [template, setTemplate] = useState("blank")
  const [submitting, setSubmitting] = useState(false)

  useEffect(() => {
    if (phpVersions.length > 0 && !phpVersions.includes(phpVersion)) {
      setPhpVersion(phpVersions[0])
    }
  }, [phpVersions])

  const handleNameChange = (value: string) => {
    setName(value)
    if (!domain || domain === name.toLowerCase().replace(/\s+/g, "-") + ".test") {
      setDomain(value.toLowerCase().replace(/\s+/g, "-") + ".test")
    }
  }

  const handleSubmit = async () => {
    if (!name || !domain || !documentRoot) return
    setSubmitting(true)
    try {
      await onSubmit(name, domain, documentRoot, phpVersion, ssl, template === "blank" ? undefined : template)
      setOpen(false)
      setName("")
      setDomain("")
      setDocumentRoot("")
      setPhpVersion(phpVersions[0] ?? "")
      setSsl(false)
      setTemplate("blank")
    } finally {
      setSubmitting(false)
    }
  }

  return (
    <Dialog open={open} onOpenChange={setOpen}>
      <DialogTrigger asChild>
        <Button size="sm">
          <Plus className="mr-1.5 h-3.5 w-3.5" />
          Add Site
        </Button>
      </DialogTrigger>
      <DialogContent>
        <DialogHeader>
          <DialogTitle>Create New Site</DialogTitle>
        </DialogHeader>
        <div className="space-y-4 pt-2">
          <div className="space-y-1.5">
            <Label htmlFor="name">Site Name</Label>
            <Input
              id="name"
              placeholder="My Project"
              value={name}
              onChange={(e) => handleNameChange(e.target.value)}
            />
          </div>
          <div className="space-y-1.5">
            <Label>Template</Label>
            <Select value={template} onValueChange={setTemplate}>
              <SelectTrigger>
                <SelectValue />
              </SelectTrigger>
              <SelectContent>
                <SelectItem value="blank">Blank Site</SelectItem>
                <SelectItem value="wordpress">WordPress</SelectItem>
                <SelectItem value="laravel">Laravel</SelectItem>
                <SelectItem value="fatfree">Fat-Free Framework</SelectItem>
              </SelectContent>
            </Select>
          </div>
          <div className="space-y-1.5">
            <Label htmlFor="domain">Domain</Label>
            <Input
              id="domain"
              placeholder="myproject.test"
              value={domain}
              onChange={(e) => setDomain(e.target.value)}
            />
          </div>
          <div className="space-y-1.5">
            <Label htmlFor="docRoot">Document Root</Label>
            <div className="flex gap-2">
              <Input
                id="docRoot"
                placeholder="/Users/you/Sites/myproject"
                value={documentRoot}
                onChange={(e) => setDocumentRoot(e.target.value)}
                className="flex-1"
              />
              <Button
                type="button"
                size="icon"
                variant="outline"
                onClick={async () => {
                  const selected = await openDialog({
                    directory: true,
                    title: "Select Document Root",
                  })
                  if (selected) setDocumentRoot(selected)
                }}
              >
                <FolderOpen className="h-4 w-4" />
              </Button>
            </div>
          </div>
          <div className="space-y-1.5">
            <Label>PHP Version</Label>
            {phpVersions.length === 0 ? (
              <p className="text-xs text-muted-foreground">No PHP versions installed. Install one from the PHP page first.</p>
            ) : (
              <Select value={phpVersion} onValueChange={setPhpVersion}>
                <SelectTrigger>
                  <SelectValue />
                </SelectTrigger>
                <SelectContent>
                  {phpVersions.map((v) => (
                    <SelectItem key={v} value={v}>
                      PHP {v}
                    </SelectItem>
                  ))}
                </SelectContent>
              </Select>
            )}
          </div>
          <div className="flex items-center justify-between">
            <Label htmlFor="ssl">Enable SSL</Label>
            <Switch id="ssl" checked={ssl} onCheckedChange={setSsl} />
          </div>
          <Button
            className="w-full"
            onClick={handleSubmit}
            disabled={!name || !domain || !documentRoot || !phpVersion || submitting}
          >
            {submitting ? "Creating..." : "Create Site"}
          </Button>
        </div>
      </DialogContent>
    </Dialog>
  )
}
