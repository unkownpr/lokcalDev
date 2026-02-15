import { useEffect, useState } from "react"
import { listen } from "@tauri-apps/api/event"
import type { DownloadProgress } from "@/types/php"

export function useDownloadProgress() {
  const [progress, setProgress] = useState<Record<string, DownloadProgress>>({})

  useEffect(() => {
    const unlisten = listen<DownloadProgress>("download-progress", (event) => {
      setProgress((prev) => ({
        ...prev,
        [event.payload.id]: event.payload,
      }))
    })

    return () => {
      unlisten.then((fn) => fn())
    }
  }, [])

  return progress
}
