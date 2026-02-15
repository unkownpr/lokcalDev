import { useEffect, useRef } from "react"
import { ScrollText, Trash2, Play, Square } from "lucide-react"
import { Card } from "@/components/ui/card"
import { Button } from "@/components/ui/button"
import { ScrollArea } from "@/components/ui/scroll-area"
import { PageHeader } from "@/components/shared/PageHeader"
import { EmptyState } from "@/components/shared/EmptyState"
import { useLogStore } from "@/stores/logStore"

function formatSize(bytes: number): string {
  if (bytes < 1024) return `${bytes} B`
  if (bytes < 1024 * 1024) return `${(bytes / 1024).toFixed(1)} KB`
  return `${(bytes / (1024 * 1024)).toFixed(1)} MB`
}

export function LogsPage() {
  const {
    files,
    lines,
    selectedFile,
    tailing,
    fetchFiles,
    readFile,
    startTailing,
    stopTailing,
    clearFile,
    setSelectedFile,
  } = useLogStore()

  const logEndRef = useRef<HTMLDivElement>(null)

  useEffect(() => {
    fetchFiles()
  }, [fetchFiles])

  useEffect(() => {
    if (tailing && logEndRef.current) {
      logEndRef.current.scrollIntoView({ behavior: "smooth" })
    }
  }, [lines, tailing])

  return (
    <div>
      <PageHeader title="Logs" description="View service and application logs" />

      {files.length === 0 ? (
        <EmptyState
          icon={ScrollText}
          title="No log files"
          description="Log files will appear here once services start generating logs."
        />
      ) : (
        <div className="grid grid-cols-[240px_1fr] gap-4 h-[calc(100vh-200px)]">
          {/* File list */}
          <Card className="p-2 overflow-y-auto">
            <div className="space-y-0.5">
              {files.map((file) => (
                <button
                  key={file.path}
                  className={`w-full text-left px-2 py-1.5 rounded text-sm transition-colors ${
                    selectedFile === file.path
                      ? "bg-accent text-accent-foreground"
                      : "hover:bg-muted/50"
                  }`}
                  onClick={() => {
                    readFile(file.path)
                    setSelectedFile(file.path)
                  }}
                >
                  <p className="truncate font-medium text-xs">{file.name}</p>
                  <p className="text-[10px] text-muted-foreground">{formatSize(file.size)}</p>
                </button>
              ))}
            </div>
          </Card>

          {/* Log content */}
          <Card className="flex flex-col overflow-hidden">
            {selectedFile ? (
              <>
                <div className="flex items-center justify-between px-3 py-2 border-b">
                  <span className="text-xs font-medium truncate">
                    {files.find((f) => f.path === selectedFile)?.name}
                  </span>
                  <div className="flex gap-1">
                    {!tailing ? (
                      <Button
                        size="sm"
                        variant="ghost"
                        className="h-7 text-xs"
                        onClick={() => startTailing(selectedFile)}
                      >
                        <Play className="mr-1 h-3 w-3" /> Tail
                      </Button>
                    ) : (
                      <Button
                        size="sm"
                        variant="ghost"
                        className="h-7 text-xs"
                        onClick={stopTailing}
                      >
                        <Square className="mr-1 h-3 w-3" /> Stop
                      </Button>
                    )}
                    <Button
                      size="sm"
                      variant="ghost"
                      className="h-7 text-xs text-destructive"
                      onClick={() => clearFile(selectedFile)}
                    >
                      <Trash2 className="mr-1 h-3 w-3" /> Clear
                    </Button>
                  </div>
                </div>
                <ScrollArea className="flex-1 p-3">
                  <pre className="text-[11px] font-mono leading-relaxed text-muted-foreground">
                    {lines.length === 0 ? (
                      <span className="text-muted-foreground/50">Empty log file</span>
                    ) : (
                      lines.map((line, i) => (
                        <div key={i} className="hover:bg-muted/30 px-1">
                          {line}
                        </div>
                      ))
                    )}
                    <div ref={logEndRef} />
                  </pre>
                </ScrollArea>
              </>
            ) : (
              <div className="flex items-center justify-center h-full text-sm text-muted-foreground">
                Select a log file to view
              </div>
            )}
          </Card>
        </div>
      )}
    </div>
  )
}
