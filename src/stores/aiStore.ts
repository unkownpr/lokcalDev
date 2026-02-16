import { create } from "zustand"
import { listen, type UnlistenFn } from "@tauri-apps/api/event"
import type { AiMessage, AiStreamChunk, AiToolCall } from "@/types/ai"
import * as tauri from "@/lib/tauri"

interface AiStore {
  messages: AiMessage[]
  streaming: boolean
  error: string | null
  unlisten: UnlistenFn | null

  sendMessage: (content: string) => Promise<void>
  executeToolCalls: (toolCalls: AiToolCall[]) => Promise<void>
  clearMessages: () => void
  cleanup: () => void
}

export const useAiStore = create<AiStore>((set, get) => ({
  messages: [],
  streaming: false,
  error: null,
  unlisten: null,

  sendMessage: async (content: string) => {
    const { messages, unlisten: prevUnlisten } = get()

    // Cleanup previous listener
    if (prevUnlisten) {
      prevUnlisten()
    }

    // Add user message
    const userMessage: AiMessage = { role: "user", content }
    const updatedMessages = [...messages, userMessage]
    set({ messages: updatedMessages, streaming: true, error: null })

    // Add placeholder assistant message
    const assistantMessage: AiMessage = { role: "assistant", content: "" }
    set({ messages: [...updatedMessages, assistantMessage] })

    // Listen for stream chunks
    let accumulatedContent = ""
    let receivedToolCalls: AiToolCall[] | null = null

    const unlisten = await listen<AiStreamChunk>("ai-stream", (event) => {
      const chunk = event.payload

      switch (chunk.chunkType) {
        case "content":
          accumulatedContent += chunk.content ?? ""
          set((state) => {
            const msgs = [...state.messages]
            const lastIdx = msgs.length - 1
            if (lastIdx >= 0 && msgs[lastIdx].role === "assistant") {
              msgs[lastIdx] = { ...msgs[lastIdx], content: accumulatedContent }
            }
            return { messages: msgs }
          })
          break

        case "tool_calls":
          receivedToolCalls = chunk.toolCalls ?? null
          set((state) => {
            const msgs = [...state.messages]
            const lastIdx = msgs.length - 1
            if (lastIdx >= 0 && msgs[lastIdx].role === "assistant") {
              msgs[lastIdx] = {
                ...msgs[lastIdx],
                content: accumulatedContent || null,
                toolCalls: receivedToolCalls ?? undefined,
              }
            }
            return { messages: msgs }
          })
          break

        case "done":
          set({ streaming: false })
          unlisten()
          set({ unlisten: null })

          // If there were tool calls, execute them
          if (receivedToolCalls && receivedToolCalls.length > 0) {
            get().executeToolCalls(receivedToolCalls)
          }
          break

        case "error":
          set({ streaming: false, error: chunk.error ?? "Unknown error" })
          unlisten()
          set({ unlisten: null })
          break
      }
    })

    set({ unlisten })

    // Send to backend
    try {
      await tauri.aiChat(updatedMessages)
    } catch (err) {
      set({ streaming: false, error: String(err) })
      unlisten()
      set({ unlisten: null })
    }
  },

  executeToolCalls: async (toolCalls: AiToolCall[]) => {
    const { messages } = get()

    // Execute each tool call and collect results
    const toolMessages: AiMessage[] = []
    for (const tc of toolCalls) {
      try {
        const result = await tauri.aiExecuteTool(tc.function.name, tc.function.arguments)
        toolMessages.push({
          role: "tool",
          content: result,
          toolCallId: tc.id,
        })
      } catch (err) {
        toolMessages.push({
          role: "tool",
          content: `Error: ${String(err)}`,
          toolCallId: tc.id,
        })
      }
    }

    // Add tool results to messages
    const messagesWithTools = [...messages, ...toolMessages]
    set({ messages: messagesWithTools, streaming: true })

    // Add placeholder for next assistant response
    const assistantMessage: AiMessage = { role: "assistant", content: "" }
    set({ messages: [...messagesWithTools, assistantMessage] })

    // Listen for the continuation
    let accumulatedContent = ""
    let receivedToolCalls: AiToolCall[] | null = null

    const unlisten = await listen<AiStreamChunk>("ai-stream", (event) => {
      const chunk = event.payload

      switch (chunk.chunkType) {
        case "content":
          accumulatedContent += chunk.content ?? ""
          set((state) => {
            const msgs = [...state.messages]
            const lastIdx = msgs.length - 1
            if (lastIdx >= 0 && msgs[lastIdx].role === "assistant") {
              msgs[lastIdx] = { ...msgs[lastIdx], content: accumulatedContent }
            }
            return { messages: msgs }
          })
          break

        case "tool_calls":
          receivedToolCalls = chunk.toolCalls ?? null
          set((state) => {
            const msgs = [...state.messages]
            const lastIdx = msgs.length - 1
            if (lastIdx >= 0 && msgs[lastIdx].role === "assistant") {
              msgs[lastIdx] = {
                ...msgs[lastIdx],
                content: accumulatedContent || null,
                toolCalls: receivedToolCalls ?? undefined,
              }
            }
            return { messages: msgs }
          })
          break

        case "done":
          set({ streaming: false })
          unlisten()
          set({ unlisten: null })

          if (receivedToolCalls && receivedToolCalls.length > 0) {
            get().executeToolCalls(receivedToolCalls)
          }
          break

        case "error":
          set({ streaming: false, error: chunk.error ?? "Unknown error" })
          unlisten()
          set({ unlisten: null })
          break
      }
    })

    set({ unlisten })

    // Send continuation to backend
    try {
      await tauri.aiChat(messagesWithTools)
    } catch (err) {
      set({ streaming: false, error: String(err) })
      unlisten()
      set({ unlisten: null })
    }
  },

  clearMessages: () => {
    const { unlisten } = get()
    if (unlisten) unlisten()
    set({ messages: [], streaming: false, error: null, unlisten: null })
  },

  cleanup: () => {
    const { unlisten } = get()
    if (unlisten) unlisten()
    set({ unlisten: null })
  },
}))
