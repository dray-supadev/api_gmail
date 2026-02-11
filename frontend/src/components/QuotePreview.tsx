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
    pdfBase64?: string
    pdfName?: string
    className?: string
    company?: string
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
    pdfBase64,
    pdfName,
    className,
    company
}: QuotePreviewProps) {
    const [comment, setComment] = useState("")
    const [to, setTo] = useState(initialTo.join(", "))
    const [cc, setCc] = useState("")
    const [subject, setSubject] = useState(initialSubject)
    const [sending, setSending] = useState(false)
    const [loadingPreview, setLoadingPreview] = useState(false)

    const [templateHtml, setTemplateHtml] = useState<string>("")

    // Derived state for the actual HTML in the iframe
    const previewHtml = templateHtml.replace(/<comment>/g, comment.replace(/\n/g, "<br>"))

    // Fetch template HTML (only when quote or settings change)
    useEffect(() => {
        const fetchPreview = async () => {
            setLoadingPreview(true)
            try {
                const res = await api.previewQuote({
                    quote_id: quoteId,
                    version,
                    comment: "",
                    pdf_export_settings: pdfExportSettings
                })
                setTemplateHtml(res.html)
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

        fetchPreview()
    }, [quoteId, version, pdfExportSettings])

    const handleSend = async () => {
        setSending(true)
        try {
            // Generate maildata_Identificator: "DI" + 4 random alphanumeric chars
            const randomChars = Math.random().toString(36).substring(2, 6).toUpperCase();
            const maildata_identificator = `DI${randomChars}`;

            await api.sendQuote(token, {
                quote_id: quoteId,
                version,
                provider,
                to: to.split(",").map(s => s.trim()).filter(Boolean),
                cc: cc.split(",").map(s => s.trim()).filter(Boolean),
                subject,
                thread_id: threadId,
                comment,
                pdf_export_settings: pdfExportSettings,
                html_body: previewHtml,
                pdf_base64: pdfBase64,
                pdf_name: pdfName,
                maildata_identificator: maildata_identificator,
                company
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

                    <div className="grid grid-cols-[60px_1fr] items-center gap-2">
                        <label className="text-sm font-medium text-muted-foreground text-right">CC</label>
                        <input
                            className="flex h-9 w-full rounded-md border border-input bg-transparent px-3 py-1 text-sm shadow-sm transition-colors focus-visible:outline-none focus-visible:ring-1 focus-visible:ring-ring"
                            value={cc}
                            onChange={e => setCc(e.target.value)}
                            placeholder="cc@example.com"
                        />
                    </div>

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
