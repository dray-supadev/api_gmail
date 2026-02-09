import { useState, useRef, useEffect } from "react"
import { Search, FolderInput } from "lucide-react"
import { cn } from "@/lib/utils"

import type { Message, Label } from "../api"

interface MessageListProps {
    messages: Message[]
    selectedId: string | null
    onSelect: (id: string) => void
    onSearch: (query: string) => void
    labelName?: string
    labels?: Label[]
    onMove?: (messageId: string, newLabelId: string) => void
}

export function MessageList({ messages, selectedId, onSelect, onSearch, labelName = "Inbox", labels = [], onMove }: MessageListProps) {
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
                    <MessageItem
                        key={message.id}
                        message={message}
                        isSelected={selectedId === message.id}
                        onSelect={onSelect}
                        labels={labels}
                        onMove={onMove}
                    />
                ))}
            </div>
        </div>
    )
}

interface MessageItemProps {
    message: Message
    isSelected: boolean
    onSelect: (id: string) => void
    labels: Label[]
    onMove?: (messageId: string, newLabelId: string) => void
}

function MessageItem({ message, isSelected, onSelect, labels, onMove }: MessageItemProps) {
    const [showMoveMenu, setShowMoveMenu] = useState(false)
    const menuRef = useRef<HTMLDivElement>(null)

    // Close menu when clicking outside
    useEffect(() => {
        function handleClickOutside(event: MouseEvent) {
            if (menuRef.current && !menuRef.current.contains(event.target as Node)) {
                setShowMoveMenu(false)
            }
        }
        document.addEventListener("mousedown", handleClickOutside)
        return () => document.removeEventListener("mousedown", handleClickOutside)
    }, [])

    const handleMoveClick = (e: React.MouseEvent, labelId: string) => {
        e.stopPropagation() // Prevent selecting the message
        onMove?.(message.id, labelId)
        setShowMoveMenu(false)
    }

    return (
        <div
            onClick={() => onSelect(message.id)}
            className={cn(
                "relative group p-4 border-b cursor-pointer hover:bg-accent/50 transition-colors",
                isSelected ? "bg-accent border-l-4 border-l-primary" : "border-l-4 border-l-transparent",
                message.unread ? "font-semibold" : "text-muted-foreground"
            )}
        >
            {/* Hover Actions */}
            <div className={cn(
                "absolute top-2 right-2 flex space-x-1 opacity-0 group-hover:opacity-100 transition-opacity",
                showMoveMenu ? "opacity-100" : ""
            )}>
                <div className="relative" ref={menuRef}>
                    <button
                        onClick={(e) => {
                            e.stopPropagation()
                            setShowMoveMenu(!showMoveMenu)
                        }}
                        className="p-1.5 rounded-full hover:bg-background text-muted-foreground hover:text-foreground shadow-sm bg-white/80 backdrop-blur-sm border"
                        title="Move to..."
                    >
                        <FolderInput className="h-4 w-4" />
                    </button>

                    {/* Move Menu Dropdown */}
                    {showMoveMenu && (
                        <div className="absolute right-0 mt-1 w-48 bg-popover text-popover-foreground border rounded-md shadow-lg z-50 py-1 max-h-60 overflow-y-auto">
                            <div className="px-2 py-1.5 text-xs font-semibold text-muted-foreground">Move to...</div>
                            {labels.map((label) => (
                                <button
                                    key={label.id}
                                    onClick={(e) => handleMoveClick(e, label.id)}
                                    className="w-full text-left px-2 py-1.5 text-sm hover:bg-accent hover:text-accent-foreground flex items-center"
                                >
                                    <span className="truncate">{label.name}</span>
                                </button>
                            ))}
                        </div>
                    )}
                </div>
            </div>

            <div className="flex justify-between items-start mb-1">
                <span className={cn("text-sm truncate max-w-[200px]", message.unread ? "text-foreground" : "")}>
                    {message.from || "Unknown"}
                </span>
                <span className="text-xs text-muted-foreground whitespace-nowrap ml-2">
                    {message.date || ""}
                </span>
            </div>
            <h3 className={cn("text-sm mb-1 truncate pr-6", message.unread ? "text-foreground" : "")}>
                {message.subject || "(No Subject)"}
            </h3>
            <p className="text-xs text-muted-foreground line-clamp-2">
                {message.snippet || ""}
            </p>
        </div>
    )
}
