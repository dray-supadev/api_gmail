import { useState, useEffect } from "react"
import { api } from "@/api"
import { Button } from "./ui/button"
import { cn } from "@/lib/utils"

interface QuotePreviewProps {
    quoteId: string
    version?: string
    initialTo: string[]
    initialSubject: string
    threadId?: string
    token: string
    provider: string
    onClose: () => void
    className?: string
}

export function QuotePreview({
    quoteId,
    version,
    initialTo,
    initialSubject,
    threadId,
    token,
    provider,
    onClose,
    className
}: QuotePreviewProps) {
    const [html, setHtml] = useState<string>("")
    const [comment, setComment] = useState("")
    const [to, setTo] = useState(initialTo.join(", "))
    const [subject, setSubject] = useState(initialSubject)
    const [sending, setSending] = useState(false)
    const [loadingPreview, setLoadingPreview] = useState(false)

    // Debounced preview update could be better, but for now fetch on effect
    useEffect(() => {
        const fetchPreview = async () => {
            setLoadingPreview(true)
            try {
                const res = await api.previewQuote({
                    quote_id: quoteId,
                    version,
                    comment
                })
                setHtml(res.html)
            } catch (e) {
                console.error(e)
                setHtml("<p class='text-red-500'>Failed to load preview</p>")
            } finally {
                setLoadingPreview(false)
            }
        }

        // Simple debounce
        const timer = setTimeout(fetchPreview, 500)
        return () => clearTimeout(timer)
    }, [quoteId, version, comment])

    const handleSend = async () => {
        setSending(true)
        try {
            await api.sendQuote(token, {
                quote_id: quoteId,
                version,
                provider,
                to: to.split(",").map(s => s.trim()).filter(Boolean),
                subject,
                thread_id: threadId,
                comment
            })
            alert("Quote sent successfully!")
            onClose()
        } catch (e) {
            console.error(e)
            alert("Failed to send quote")
        } finally {
            setSending(false)
        }
    }

    return (
        <div className={cn("flex flex-col md:flex-row bg-background h-full shadow-xl z-10 overflow-hidden", className)}>
            {/* Left Panel: Form & Controls */}
            <div className="w-full md:w-[400px] flex flex-col border-r bg-card z-20 shadow-sm">
                <div className="p-6 border-b flex justify-between items-start">
                    <div>
                        <h2 className="font-semibold text-xl tracking-tight">Send Quote</h2>
                        <p className="text-sm text-muted-foreground mt-1">Compose your quote proposal.</p>
                    </div>
                    <button onClick={onClose} className="text-muted-foreground hover:text-foreground p-1 rounded-md hover:bg-muted transition-colors">
                        <span className="sr-only">Close</span>
                        <svg className="w-5 h-5" fill="none" viewBox="0 0 24 24" stroke="currentColor"><path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M6 18L18 6M6 6l12 12" /></svg>
                    </button>
                </div>

                <div className="p-6 space-y-5 overflow-y-auto flex-1">
                    <div className="space-y-2">
                        <label className="text-sm font-medium leading-none peer-disabled:cursor-not-allowed peer-disabled:opacity-70">To</label>
                        <input
                            className="flex h-10 w-full rounded-md border border-input bg-transparent px-3 py-2 text-sm shadow-sm transition-colors focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring"
                            value={to}
                            onChange={e => setTo(e.target.value)}
                            placeholder="recipient@example.com"
                        />
                    </div>

                    <div className="space-y-2">
                        <label className="text-sm font-medium leading-none">Subject</label>
                        <input
                            className="flex h-10 w-full rounded-md border border-input bg-transparent px-3 py-2 text-sm shadow-sm transition-colors focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring"
                            value={subject}
                            onChange={e => setSubject(e.target.value)}
                        />
                    </div>

                    <div className="space-y-2">
                        <label className="text-sm font-medium leading-none">Comment</label>
                        <textarea
                            className="flex min-h-[120px] w-full rounded-md border border-input bg-transparent px-3 py-2 text-sm shadow-sm placeholder:text-muted-foreground focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring disabled:cursor-not-allowed disabled:opacity-50"
                            value={comment}
                            onChange={e => setComment(e.target.value)}
                            placeholder="Add a personalized message..."
                        />
                    </div>
                </div>

                <div className="p-6 border-t bg-muted/10 flex justify-between items-center gap-3">
                    <Button variant="outline" onClick={onClose} className="flex-1">Cancel</Button>
                    <Button onClick={handleSend} disabled={sending} className="flex-1 bg-blue-600 hover:bg-blue-700 text-white">
                        {sending ? "Sending..." : "Send Quote"}
                    </Button>
                </div>
            </div>

            {/* Right Panel: Live Preview */}
            <div className="flex-1 bg-slate-50 relative flex flex-col h-full overflow-hidden">
                <div className="absolute inset-0 p-8 flex flex-col">
                    <div className="flex items-center justify-between mb-4 px-2">
                        <h3 className="font-medium text-muted-foreground text-sm uppercase tracking-wider">Live Preview</h3>
                        {loadingPreview && <span className="text-xs text-blue-600 animate-pulse font-medium">Updating...</span>}
                    </div>

                    <div className="flex-1 rounded-xl border shadow-sm bg-white overflow-hidden relative">
                        {loadingPreview && html === "" && (
                            <div className="absolute inset-0 flex items-center justify-center bg-white/50 backdrop-blur-sm z-10">
                                <div className="h-6 w-6 border-2 border-blue-600 border-t-transparent rounded-full animate-spin"></div>
                            </div>
                        )}
                        <iframe
                            title="preview"
                            srcDoc={html || "<div style='display:flex;height:100%;align-items:center;justify-content:center;color:#999;font-family:sans-serif;'>Generating preview...</div>"}
                            className="w-full h-full border-none"
                        />
                    </div>
                </div>
            </div>
        </div>
    )
}
