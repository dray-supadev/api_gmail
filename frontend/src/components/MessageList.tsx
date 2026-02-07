import { useState } from "react"
import { Search } from "lucide-react"
import { cn } from "@/lib/utils"

import type { Message } from "../api"

interface MessageListProps {
    messages: Message[]
    selectedId: string | null
    onSelect: (id: string) => void
    onSearch: (query: string) => void
    labelName?: string
}

export function MessageList({ messages, selectedId, onSelect, onSearch, labelName = "Inbox" }: MessageListProps) {
    const [searchText, setSearchText] = useState("")
    if (!messages.length && !searchText) {
        return (
            <div className="flex flex-col h-full border-r bg-background w-80 lg:w-96">
                <div className="p-4 border-b space-y-4">
                    <h1 className="text-xl font-bold">{labelName}</h1>
                </div>
                <div className="p-10 text-center text-muted-foreground flex-1">No messages in {labelName}</div>
            </div>
        )
    }

    return (
        <div className="flex flex-col h-full border-r bg-background w-[320px]">
            {/* Header / Search */}
            <div className="p-4 border-b space-y-4">
                <h1 className="text-xl font-bold">{labelName}</h1>
                <div className="relative">
                    <Search className="absolute left-2.5 top-2.5 h-4 w-4 text-muted-foreground" />
                    <input
                        type="text"
                        placeholder="Search emails..."
                        className="w-full bg-secondary text-sm pl-9 pr-4 py-2 rounded-md outline-none focus:ring-1 focus:ring-ring"
                        value={searchText}
                        onChange={(e) => setSearchText(e.target.value)}
                        onKeyDown={(e) => {
                            if (e.key === 'Enter') {
                                onSearch(searchText);
                            }
                        }}
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
