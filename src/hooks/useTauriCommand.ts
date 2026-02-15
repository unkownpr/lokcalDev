import { useCallback, useEffect, useRef, useState } from "react"

interface UseTauriCommandResult<T> {
  data: T | null
  error: string | null
  loading: boolean
  refetch: () => void
}

export function useTauriCommand<T>(
  command: () => Promise<T>,
  immediate = true,
): UseTauriCommandResult<T> {
  const [data, setData] = useState<T | null>(null)
  const [error, setError] = useState<string | null>(null)
  const [loading, setLoading] = useState(immediate)
  const commandRef = useRef(command)
  commandRef.current = command

  const execute = useCallback(async () => {
    setLoading(true)
    setError(null)
    try {
      const result = await commandRef.current()
      setData(result)
    } catch (err) {
      setError(err instanceof Error ? err.message : String(err))
    } finally {
      setLoading(false)
    }
  }, [])

  useEffect(() => {
    if (immediate) {
      execute()
    }
  }, [execute, immediate])

  return { data, error, loading, refetch: execute }
}
