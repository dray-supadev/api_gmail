import { Search } from "lucide-react"
import { cn } from "@/lib/utils"

interface Message {
    id: string
    from?: string
    subject?: string
    snippet?: string
    date?: string
    unread: boolean
    has_attachments: boolean
}

interface MessageListProps {
    messages: Message[]
    selectedId: string | null
    onSelect: (id: string) => void
}

export function MessageList({ messages, selectedId, onSelect }: MessageListProps) {
    if (!messages.length) {
        return <div className="p-4 text-center text-muted-foreground">No messages</div>
    }

    return (
        <div className="flex flex-col h-full border-r bg-background w-[400px]">
            {/* Header / Search */}
            <div className="p-4 border-b space-y-4">
                <h1 className="text-xl font-bold">Inbox</h1>
                <div className="relative">
                    <Search className="absolute left-2.5 top-2.5 h-4 w-4 text-muted-foreground" />
                    <input
                        type="text"
                        placeholder="Search emails..."
                        className="w-full bg-secondary text-sm pl-9 pr-4 py-2 rounded-md outline-none focus:ring-1 focus:ring-ring"
                    />
                </div>
            </div>

            {/* List */}
            <div className="flex-1 overflow-auto">
                {messages.map((message) => (
                    <div
                        key={message.id}
                        onClick={() => onSelect(message.id)}
                        className={cn(
                            "p-4 border-b cursor-pointer hover:bg-accent/50 transition-colors",
                            selectedId === message.id ? "bg-accent border-l-4 border-l-primary" : "border-l-4 border-l-transparent",
                            message.unread ? "font-semibold" : "text-muted-foreground"
                        )}
                    >
                        <div className="flex justify-between items-start mb-1">
                            <span className={cn("text-sm truncate max-w-[200px]", message.unread ? "text-foreground" : "")}>
                                {message.from || "Unknown"}
                            </span>
                            <span className="text-xs text-muted-foreground whitespace-nowrap ml-2">
                                {message.date || ""}
                            </span>
                        </div>
                        <h3 className={cn("text-sm mb-1 truncate", message.unread ? "text-foreground" : "")}>
                            {message.subject || "(No Subject)"}
                        </h3>
                        <p className="text-xs text-muted-foreground line-clamp-2">
                            {message.snippet || ""}
                        </p>
                    </div>
                ))}
            </div>
        </div>
    )
}
