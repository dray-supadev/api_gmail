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
        <div className={cn("w-[450px] border-l flex flex-col bg-background h-full shadow-xl z-10", className)}>
            <div className="p-4 border-b flex justify-between items-center bg-muted/30">
                <h2 className="font-semibold text-lg">Send Quote</h2>
                {/* Close button removed to avoid confusion with main widget close button */}
            </div>

            <div className="p-4 space-y-4 overflow-auto flex-1">
                <div className="space-y-2">
                    <label className="text-sm font-medium">To</label>
                    <input
                        className="flex h-9 w-full rounded-md border border-input bg-transparent px-3 py-1 text-sm shadow-sm transition-colors focus-visible:outline-none focus-visible:ring-1 focus-visible:ring-ring"
                        value={to}
                        onChange={e => setTo(e.target.value)}
                    />
                </div>

                <div className="space-y-2">
                    <label className="text-sm font-medium">Subject</label>
                    <input
                        className="flex h-9 w-full rounded-md border border-input bg-transparent px-3 py-1 text-sm shadow-sm transition-colors focus-visible:outline-none focus-visible:ring-1 focus-visible:ring-ring"
                        value={subject}
                        onChange={e => setSubject(e.target.value)}
                    />
                </div>

                <div className="space-y-2">
                    <label className="text-sm font-medium">Comment (Added to quote)</label>
                    <textarea
                        className="flex min-h-[80px] w-full rounded-md border border-input bg-transparent px-3 py-2 text-sm shadow-sm placeholder:text-muted-foreground focus-visible:outline-none focus-visible:ring-1 focus-visible:ring-ring disabled:cursor-not-allowed disabled:opacity-50"
                        value={comment}
                        onChange={e => setComment(e.target.value)}
                        placeholder="Add a comment to the quote..."
                    />
                </div>

                <div className="space-y-2">
                    <label className="text-sm font-medium">Preview</label>
                    <div className="border rounded-md overflow-hidden bg-white min-h-[400px]">
                        {loadingPreview && <div className="p-4 text-center text-sm text-muted-foreground">Loading preview...</div>}
                        {!loadingPreview && (
                            <iframe
                                title="preview"
                                srcDoc={html}
                                className="w-full h-[600px] border-none"
                                style={{ pointerEvents: 'none' }} // Disable interaction within preview
                            />
                        )}
                    </div>
                </div>
            </div>

            <div className="p-4 border-t bg-muted/30 flex justify-end gap-2">
                <Button variant="outline" onClick={onClose}>Cancel</Button>
                <Button onClick={handleSend} disabled={sending}>
                    {sending ? "Sending..." : "Send Quote"}
                </Button>
            </div>
        </div>
    )
}
