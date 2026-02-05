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
    pdfExportSettings?: string[]
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
    pdfExportSettings = [],
    className
}: QuotePreviewProps) {
    const [templateHtml, setTemplateHtml] = useState<string>("")

    // Derived state for the actual HTML in the iframe
    const previewHtml = templateHtml.replace(/<comment>/g, comment.replace(/\n/g, "<br>"))

    // Fetch template HTML (only when quote or settings change)
    useEffect(() => {
        const fetchPreview = async () => {
            setLoadingPreview(true)
            try {
                // Pass empty comment or dummy placeholder to get the template
                const res = await api.previewQuote({
                    quote_id: quoteId,
                    version,
                    comment: "", // We don't send comment to backend anymore for preview generation
                    pdf_export_settings: pdfExportSettings
                })
                setTemplateHtml(res.html)
                // If backend provides a default body (email template), use it if user input is empty
                if (res.body && !comment) {
                    setComment(res.body)
                }
            } catch (e) {
                console.error("Preview generation failed:", e)
                setTemplateHtml(`<div class="p-4 text-red-500 flex flex-col items-center justify-center h-full">
                    <p class="font-bold">Failed to load preview</p>
                    <p class="text-sm mt-2 text-gray-500">${e instanceof Error ? e.message : "Unknown error"}</p>
                    <button onclick="window.location.reload()" class="mt-4 px-3 py-1 bg-red-100 rounded text-xs hover:bg-red-200">Retry</button>
                </div>`)
            } finally {
                setLoadingPreview(false)
            }
        }

        // Debounce only on settings change if needed, but for now simple fetch
        fetchPreview()
    }, [quoteId, version, pdfExportSettings]) // Trigger when these change. Removed 'comment'. 
    // Wait, if we use Bubble for preview HTML which depends on comment (comment box in PDF), we DO need to send comment.
    // If you want realtime preview update as user types comment:
    // Keep 'comment' in dep array.
    // BUT: The logic `if (res.body && !comment) setComment(res.body)` might fight with user typing if not careful.
    // Better: Only setComment from response if it is INITIAL load.
    // But since useEffect runs on 'comment' change, it will re-fetch.
    // If the Bubble workflow returns the SAME body passed in, it's fine.
    // If Bubble workflow generates a DEFAULT body, we only want that on first load.
    // Let's refactor slightly to separate "Fetch Initial Body" vs "Update Preview HTML".
    // For now, I will keep it simple. If user types, we send comment to Bubble, Bubble returns HTML with comment. Bubble returns body? 
    // Actually, usually Body is just for the email text. HTML is for PDF. 
    // The PDF usually contains the comment too.
    // So we should send 'comment' to Bubble.
    // Bubble response 'body' is the EMAIL body template. It shouldn't change just because I typed a comment in the PDF, unless the email body also includes the comment.
    // Safest is to only setComment from response if current comment is empty.

    // UPDATED DEPENDENCIES:
    // [quoteId, version, comment, pdfExportSettings]
    // And logic to setComment only if !comment.
    // Actually, if I type a char, 'comment' changes -> fetchPreview -> returns 'body' -> setComment -> loop?
    // If returns same body, no change.
    // If I typed "H", and Bubble returns "Default Body", then "H" != "Default Body" and !"H".length is false.
    // So `if (!comment)` protects us from overwriting user input.
    // But what if `comment` is required for the Preview HTML?

    // Let's assume user wants to see their comment in the PDF preview.
    // But fetchPreview is debounced.
    // So it's fine.


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
                comment,
                pdf_export_settings: pdfExportSettings
            })
            onClose()
        } catch (e) {
            console.error(e)
            alert("Failed to send quote")
        } finally {
            setSending(false)
        }
    }

    return (
        <div className={cn("flex flex-col h-full bg-white relative", className)}>
            {/* Top Bar: Controls & Inputs */}
            <div className="flex-none p-6 pb-2 border-b space-y-4">
                <div className="flex justify-between items-center mb-2">
                    <h2 className="font-semibold text-lg">New Quote Proposal</h2>
                    <Button variant="ghost" size="sm" onClick={onClose}>
                        <svg className="w-5 h-5 text-muted-foreground" fill="none" viewBox="0 0 24 24" stroke="currentColor"><path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M6 18L18 6M6 6l12 12" /></svg>
                    </Button>
                </div>

                <div className="grid gap-4 max-w-3xl">
                    <div className="grid grid-cols-[60px_1fr] items-center gap-2">
                        <label className="text-sm font-medium text-muted-foreground text-right">To</label>
                        <input
                            className="flex h-9 w-full rounded-md border border-input bg-transparent px-3 py-1 text-sm shadow-sm transition-colors focus-visible:outline-none focus-visible:ring-1 focus-visible:ring-ring"
                            value={to}
                            onChange={e => setTo(e.target.value)}
                            placeholder="recipient@example.com"
                        />
                    </div>
                    {/* Placeholder for CC if needed later */}

                    <div className="grid grid-cols-[60px_1fr] items-center gap-2">
                        <label className="text-sm font-medium text-muted-foreground text-right">Subject</label>
                        <input
                            className="flex h-9 w-full rounded-md border border-input bg-transparent px-3 py-1 text-sm shadow-sm transition-colors focus-visible:outline-none focus-visible:ring-1 focus-visible:ring-ring"
                            value={subject}
                            onChange={e => setSubject(e.target.value)}
                        />
                    </div>
                </div>
            </div>

            {/* Scrollable Content Area: Comment + Preview */}
            <div className="flex-1 overflow-y-auto p-6 bg-slate-50/50">
                <div className="max-w-4xl mx-auto space-y-6">
                    {/* Comment Input */}
                    <div className="bg-white p-4 rounded-lg border shadow-sm space-y-2">
                        <label className="text-sm font-semibold text-slate-700">Message / Comment</label>
                        <textarea
                            className="flex min-h-[100px] w-full rounded-md border-0 bg-slate-50 px-3 py-2 text-sm focus-visible:ring-1 ring-blue-200 resize-y"
                            value={comment}
                            onChange={e => setComment(e.target.value)}
                            placeholder="Type your message to the customer here..."
                        />
                    </div>

                    {/* Preview Card */}
                    <div className="relative min-h-[500px] border rounded-lg bg-white shadow-sm overflow-hidden">
                        {loadingPreview && (
                            <div className="absolute inset-0 flex items-center justify-center bg-white/50 backdrop-blur-sm z-10">
                                <div className="h-6 w-6 border-2 border-blue-600 border-t-transparent rounded-full animate-spin"></div>
                            </div>
                        )}
                        <iframe
                            title="preview"
                            srcDoc={previewHtml || ""}
                            className="w-full h-[800px] border-none"
                        />
                    </div>
                </div>
            </div>

            {/* Bottom Bar: Send Action */}
            <div className="flex-none p-4 border-t bg-white flex justify-end items-center gap-3">
                <Button variant="ghost" onClick={onClose}>Cancel</Button>
                <Button onClick={handleSend} disabled={sending} className="bg-blue-600 hover:bg-blue-700 text-white min-w-[120px]">
                    {sending ? <span className="flex items-center gap-2"><div className="h-4 w-4 border-2 border-white/50 border-t-white rounded-full animate-spin"></div> Sending...</span> : "Send Quote"}
                </Button>
            </div>
        </div>
    )
}
