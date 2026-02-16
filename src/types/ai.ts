export interface AiModel {
  id: string
  name: string
}

export interface AiMessage {
  role: "user" | "assistant" | "tool"
  content: string | null
  toolCalls?: AiToolCall[]
  toolCallId?: string
}

export interface AiToolCall {
  id: string
  type: string
  function: {
    name: string
    arguments: string
  }
}

export interface AiStreamChunk {
  chunkType: "content" | "tool_calls" | "done" | "error"
  content?: string
  toolCalls?: AiToolCall[]
  error?: string
}

export interface AiToolResult {
  toolCallId: string
  name: string
  result: string
  isError: boolean
}
