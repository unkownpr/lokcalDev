import { Component, type ErrorInfo, type ReactNode } from "react"
import { AlertTriangle, RotateCcw, RefreshCw } from "lucide-react"
import { Button } from "@/components/ui/button"
import { Card } from "@/components/ui/card"

interface Props {
  children: ReactNode
}

interface State {
  hasError: boolean
  error: Error | null
}

export class ErrorBoundary extends Component<Props, State> {
  constructor(props: Props) {
    super(props)
    this.state = { hasError: false, error: null }
  }

  static getDerivedStateFromError(error: Error): State {
    return { hasError: true, error }
  }

  componentDidCatch(error: Error, info: ErrorInfo) {
    console.error("ErrorBoundary caught:", error, info.componentStack)
  }

  render() {
    if (this.state.hasError) {
      return (
        <div className="flex items-center justify-center h-full p-8">
          <Card className="max-w-md w-full p-6 text-center space-y-4">
            <AlertTriangle className="h-10 w-10 text-destructive mx-auto" />
            <div>
              <h2 className="text-lg font-semibold">Something went wrong</h2>
              <p className="text-sm text-muted-foreground mt-1">
                An unexpected error occurred in this page.
              </p>
            </div>
            {this.state.error && (
              <pre className="text-xs text-left bg-muted p-3 rounded overflow-auto max-h-32">
                {this.state.error.message}
              </pre>
            )}
            <div className="flex gap-2 justify-center">
              <Button
                variant="outline"
                size="sm"
                onClick={() => this.setState({ hasError: false, error: null })}
              >
                <RotateCcw className="mr-1.5 h-3.5 w-3.5" />
                Try Again
              </Button>
              <Button
                size="sm"
                onClick={() => window.location.reload()}
              >
                <RefreshCw className="mr-1.5 h-3.5 w-3.5" />
                Reload App
              </Button>
            </div>
          </Card>
        </div>
      )
    }

    return this.props.children
  }
}
