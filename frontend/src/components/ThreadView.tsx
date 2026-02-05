import { Button } from "@/components/ui/button"
import { Reply, MoreVertical, Archive, Star } from "lucide-react"

interface ThreadViewProps {
    threadId: string | null
}

export function ThreadView({ threadId }: ThreadViewProps) {
    if (!threadId) {
        return (
            <div className="flex-1 flex items-center justify-center text-muted-foreground bg-muted/20">
                Select a conversation to continue
            </div>
        )
    }

    return (
        <div className="flex-1 flex flex-col h-full bg-background relative">
            {/* Header */}
            <div className="h-16 border-b flex items-center justify-between px-6 bg-card">
                <div className="flex-1">
                    <h2 className="text-lg font-semibold truncate">Quote Proposal from Charlotte LOGISTICS COMPANY LLC</h2>
                </div>
                <div className="flex items-center gap-2 text-muted-foreground">
                    <Button variant="ghost" size="icon"><Star className="w-5 h-5" /></Button>
                    <Button variant="ghost" size="icon"><Archive className="w-5 h-5" /></Button>
                    <Button variant="ghost" size="icon"><MoreVertical className="w-5 h-5" /></Button>
                </div>
            </div>

            {/* Content Scroll */}
            <div className="flex-1 overflow-auto p-6 space-y-6">
                {/* Example Message 1 */}
                <div className="border rounded-lg p-6 bg-card shadow-sm">
                    {/* Custom Quote Card (like in screenshot) */}
                    <div className="bg-blue-50 dark:bg-blue-950/30 border border-blue-100 dark:border-blue-900 rounded-lg p-4 mb-6">
                        <h3 className="bg-blue-500 text-white font-medium px-4 py-2 -mx-4 -mt-4 rounded-t-lg mb-4">
                            Quote Proposal from Charlotte LOGISTICS COMPANY LLC
                        </h3>
                        <div className="grid grid-cols-2 gap-4 text-sm">
                            <div>
                                <span className="font-bold block">2442 .1I</span>
                                <span className="text-muted-foreground">Origin: Fairburn, GA</span>
                            </div>
                            <div>
                                <span className="text-muted-foreground">Destination: ...</span>
                            </div>
                        </div>
                        <div className="flex gap-3 mt-6">
                            <Button className="bg-green-600 hover:bg-green-700 text-white">Accept Quote</Button>
                            <Button variant="destructive">Reject Quote</Button>
                        </div>
                    </div>

                    <div className="prose dark:prose-invert max-w-none">
                        <p>Dear Customer,</p>
                        <p>Attached is the quote you requested...</p>
                    </div>
                </div>
            </div>

            {/* Quick Reply (Hidden for now to avoid confusion with Quote Workflow) */}
            {/* <div className="p-4 border-t bg-card">
                <div className="border rounded-md p-2 bg-background focus-within:ring-2 ring-primary">
                    <textarea
                        className="w-full resize-none outline-none bg-transparent min-h-[80px]"
                        placeholder="Type your reply..."
                    />
                    <div className="flex justify-between items-center mt-2">
                        <div className="flex gap-2">
                            
                        </div>
                        <Button size="sm">
                            <Reply className="w-4 h-4 mr-2" />
                            Reply
                        </Button>
                    </div>
                </div>
            </div> */}
        </div>
    )
}
