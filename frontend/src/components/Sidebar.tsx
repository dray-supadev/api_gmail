import { useState } from "react"
import { Inbox, ChevronDown, Folder, Send, FileText, Trash2, Mail, Archive } from "lucide-react"
import { cn } from "@/lib/utils"
import type { Label } from "../api"

interface SidebarProps {
    currentProvider: "gmail" | "outlook"
    onProviderChange: (provider: "gmail" | "outlook") => void
    labels: Label[]
    selectedLabelId: string
    onLabelSelect: (id: string) => void
}

export function Sidebar({
    currentProvider,
    onProviderChange,
    labels,
    selectedLabelId,
    onLabelSelect
}: SidebarProps) {
    const [collapsed, _setCollapsed] = useState(false)

    return (
        <div className={cn("h-full bg-card border-r flex flex-col transition-all duration-300", collapsed ? "w-16" : "w-64")}>
            {/* Account Switcher */}
            <div className="p-4 border-b">
                <div className="flex items-center gap-2 cursor-pointer hover:bg-accent/50 p-2 rounded-lg" onClick={() => onProviderChange(currentProvider === "gmail" ? "outlook" : "gmail")}>
                    <div className={cn("w-8 h-8 rounded-full flex items-center justify-center text-white font-bold", currentProvider === "gmail" ? "bg-red-500" : "bg-blue-600")}>
                        {currentProvider === "gmail" ? "G" : "O"}
                    </div>
                    {!collapsed && (
                        <div className="flex-1 overflow-hidden">
                            <p className="text-sm font-medium truncate">{currentProvider === "gmail" ? "Gmail" : "Outlook"}</p>
                            <p className="text-xs text-muted-foreground truncate">Switch Account</p>
                        </div>
                    )}
                    {!collapsed && <ChevronDown className="w-4 h-4 text-muted-foreground" />}
                </div>
            </div>

            {/* Menu - Labels */}
            <nav className="flex-1 p-2 space-y-1 overflow-y-auto">
                {labels.map((label) => {
                    // Map common label names to icons
                    let Icon = Folder;
                    const name = label.name.toLowerCase();
                    const id = label.id.toUpperCase();

                    if (name.includes("inbox") || id === "INBOX") Icon = Inbox;
                    else if (name.includes("sent") || id === "SENT") Icon = Send;
                    else if (name.includes("draft") || id === "DRAFT") Icon = FileText;
                    else if (name.includes("trash") || name.includes("deleted") || id === "TRASH") Icon = Trash2;
                    else if (name.includes("spam") || name.includes("junk") || id === "SPAM") Icon = Mail;
                    else if (name.includes("archive") || id === "ARCHIVE") Icon = Archive;

                    return (
                        <div
                            key={label.id}
                            onClick={() => onLabelSelect(label.id)}
                            className={cn(
                                "flex items-center gap-3 px-3 py-2 rounded-md cursor-pointer text-sm font-medium transition-colors hover:bg-accent/50",
                                selectedLabelId === label.id ? "bg-accent text-accent-foreground" : "text-muted-foreground"
                            )}
                        >
                            <Icon className="w-5 h-5 shrink-0" />
                            {!collapsed && <span className="flex-1 truncate">{label.name}</span>}
                        </div>
                    );
                })}

                {labels.length === 0 && !collapsed && (
                    <div className="px-3 py-4 text-xs text-center text-muted-foreground italic">
                        No labels found
                    </div>
                )}
            </nav>
        </div>
    )
}
