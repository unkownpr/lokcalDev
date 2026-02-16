import { useEffect, useRef, useState } from "react"
import { useNavigate } from "react-router"
import Markdown from "react-markdown"
import { Button } from "@/components/ui/button"
import { Card } from "@/components/ui/card"
import { ScrollArea } from "@/components/ui/scroll-area"
import { PageHeader } from "@/components/shared/PageHeader"
import { useAiStore } from "@/stores/aiStore"
import { useSettingsStore } from "@/stores/settingsStore"
import {
  Bot,
  Send,
  Trash2,
  Loader2,
  Wrench,
  AlertCircle,
} from "lucide-react"
import type { AiMessage } from "@/types/ai"

const EXAMPLE_PROMPTS = [
  "List all running services",
  "Start Nginx and PHP-FPM 8.3",
  "Create a site called myapp.test",
  "Write a hello world index.php for myapp.test",
]

function ToolCallCard({ name, args }: { name: string; args: string }) {
  let parsedArgs = ""
  try {
    parsedArgs = JSON.stringify(JSON.parse(args), null, 2)
  } catch {
    parsedArgs = args
  }

  return (
    <div className="flex items-start gap-2 rounded-md border border-border bg-muted/50 px-3 py-2 text-xs">
      <Wrench className="mt-0.5 h-3.5 w-3.5 shrink-0 text-primary" />
      <div className="min-w-0">
        <span className="font-medium text-foreground">{name}</span>
        {parsedArgs && parsedArgs !== "{}" && (
          <pre className="mt-1 whitespace-pre-wrap break-all text-muted-foreground">
            {parsedArgs}
          </pre>
        )}
      </div>
    </div>
  )
}

function ToolResultCard({ content }: { content: string }) {
  return (
    <div className="rounded-md border border-border bg-card px-3 py-2 text-xs">
      <pre className="whitespace-pre-wrap break-all text-muted-foreground">
        {content}
      </pre>
    </div>
  )
}

function MessageBubble({ message }: { message: AiMessage }) {
  if (message.role === "tool") {
    return (
      <div className="flex justify-start px-4 py-1">
        <div className="max-w-[85%]">
          <ToolResultCard content={message.content ?? ""} />
        </div>
      </div>
    )
  }

  if (message.role === "user") {
    return (
      <div className="flex justify-end px-4 py-1.5">
        <div className="max-w-[85%] rounded-2xl rounded-br-sm bg-primary px-4 py-2.5 text-sm text-primary-foreground">
          {message.content}
        </div>
      </div>
    )
  }

  // Assistant
  return (
    <div className="flex justify-start gap-2.5 px-4 py-1.5">
      <div className="mt-1 flex h-7 w-7 shrink-0 items-center justify-center rounded-full bg-muted">
        <Bot className="h-4 w-4 text-muted-foreground" />
      </div>
      <div className="max-w-[85%] space-y-2">
        {message.content && (
          <div className="prose prose-sm dark:prose-invert max-w-none rounded-2xl rounded-bl-sm bg-muted px-4 py-2.5 text-sm">
            <Markdown>{message.content}</Markdown>
          </div>
        )}
        {message.toolCalls?.map((tc) => (
          <ToolCallCard key={tc.id} name={tc.function.name} args={tc.function.arguments} />
        ))}
      </div>
    </div>
  )
}

export function AiPage() {
  const { messages, streaming, error, sendMessage, clearMessages } = useAiStore()
  const { config } = useSettingsStore()
  const navigate = useNavigate()
  const [input, setInput] = useState("")
  const scrollRef = useRef<HTMLDivElement>(null)
  const textareaRef = useRef<HTMLTextAreaElement>(null)

  const hasApiKey = config?.openrouterApiKey && config.openrouterApiKey.length > 0

  // Auto-scroll to bottom
  useEffect(() => {
    if (scrollRef.current) {
      const el = scrollRef.current
      el.scrollTop = el.scrollHeight
    }
  }, [messages, streaming])

  // Fetch settings on mount
  const { fetchSettings } = useSettingsStore()
  useEffect(() => {
    fetchSettings()
  }, [fetchSettings])

  const handleSend = () => {
    const trimmed = input.trim()
    if (!trimmed || streaming) return
    setInput("")
    sendMessage(trimmed)
    // Reset textarea height
    if (textareaRef.current) {
      textareaRef.current.style.height = "auto"
    }
  }

  const handleKeyDown = (e: React.KeyboardEvent) => {
    if (e.key === "Enter" && !e.shiftKey) {
      e.preventDefault()
      handleSend()
    }
  }

  const handleTextareaInput = (e: React.ChangeEvent<HTMLTextAreaElement>) => {
    setInput(e.target.value)
    // Auto-resize
    const el = e.target
    el.style.height = "auto"
    el.style.height = Math.min(el.scrollHeight, 150) + "px"
  }

  const visibleMessages = messages.filter(
    (m) => !(m.role === "assistant" && !m.content && !m.toolCalls?.length),
  )

  return (
    <div className="flex h-full flex-col">
      <PageHeader title="AI Assistant" description={config?.aiModel ?? "Configure in Settings > AI"}>
        <Button size="sm" variant="outline" onClick={clearMessages} disabled={streaming}>
          <Trash2 className="mr-1.5 h-3.5 w-3.5" />
          Clear
        </Button>
      </PageHeader>

      {!hasApiKey ? (
        <div className="flex flex-1 items-center justify-center">
          <Card className="max-w-md p-6 text-center space-y-3">
            <AlertCircle className="mx-auto h-10 w-10 text-muted-foreground" />
            <h3 className="text-sm font-medium">API Key Required</h3>
            <p className="text-xs text-muted-foreground">
              To use the AI Assistant, add your OpenRouter API key in Settings &gt; AI tab.
            </p>
            <Button size="sm" variant="outline" onClick={() => navigate("/settings")}>
              Go to Settings
            </Button>
          </Card>
        </div>
      ) : (
        <>
          {/* Messages area */}
          <div className="relative flex-1 overflow-hidden">
            <ScrollArea className="h-full">
              <div ref={scrollRef} className="flex h-full flex-col">
                {visibleMessages.length === 0 ? (
                  <div className="flex flex-1 flex-col items-center justify-center gap-4 py-16">
                    <div className="flex h-16 w-16 items-center justify-center rounded-full bg-muted">
                      <Bot className="h-8 w-8 text-muted-foreground" />
                    </div>
                    <div className="text-center">
                      <h3 className="text-sm font-medium">How can I help?</h3>
                      <p className="mt-1 text-xs text-muted-foreground">
                        I can manage services, create sites, and write files.
                      </p>
                    </div>
                    <div className="flex flex-wrap justify-center gap-2">
                      {EXAMPLE_PROMPTS.map((prompt) => (
                        <button
                          key={prompt}
                          onClick={() => {
                            setInput(prompt)
                            textareaRef.current?.focus()
                          }}
                          className="rounded-full border border-border px-3 py-1.5 text-xs text-muted-foreground transition-colors hover:bg-muted hover:text-foreground"
                        >
                          {prompt}
                        </button>
                      ))}
                    </div>
                  </div>
                ) : (
                  <div className="space-y-1 py-4">
                    {visibleMessages.map((msg, i) => (
                      <MessageBubble key={i} message={msg} />
                    ))}
                    {streaming && (
                      <div className="flex items-center gap-2 px-4 py-2">
                        <Loader2 className="h-4 w-4 animate-spin text-muted-foreground" />
                        <span className="text-xs text-muted-foreground">Thinking...</span>
                      </div>
                    )}
                  </div>
                )}
              </div>
            </ScrollArea>
          </div>

          {/* Error banner */}
          {error && (
            <div className="mx-4 mb-2 rounded-md border border-destructive/30 bg-destructive/10 px-3 py-2 text-xs text-destructive">
              {error}
            </div>
          )}

          {/* Input area */}
          <div className="border-t border-border px-4 py-3">
            <div className="flex items-end gap-2">
              <div className="relative flex-1">
                <textarea
                  ref={textareaRef}
                  value={input}
                  onChange={handleTextareaInput}
                  onKeyDown={handleKeyDown}
                  placeholder="Ask anything about your dev environment..."
                  rows={1}
                  className="w-full resize-none rounded-lg border border-input bg-background px-3 py-2.5 text-sm placeholder:text-muted-foreground focus:outline-none focus:ring-1 focus:ring-ring"
                  disabled={streaming}
                />
              </div>
              <Button
                size="icon"
                onClick={handleSend}
                disabled={!input.trim() || streaming}
                className="h-10 w-10 shrink-0"
              >
                {streaming ? (
                  <Loader2 className="h-4 w-4 animate-spin" />
                ) : (
                  <Send className="h-4 w-4" />
                )}
              </Button>
            </div>
            <p className="mt-1.5 text-center text-[10px] text-muted-foreground">
              Enter to send, Shift+Enter for new line
            </p>
          </div>
        </>
      )}
    </div>
  )
}
