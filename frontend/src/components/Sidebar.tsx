import { useState } from "react"
import { Inbox, Folder, Send, FileText, Trash2, Mail, Archive } from "lucide-react"
import { cn } from "@/lib/utils"
import type { Label } from "../api"

interface SidebarProps {
    currentProvider: "gmail" | "outlook"
    onProviderChange: (provider: "gmail" | "outlook") => void
    labels: Label[]
    selectedLabelId: string
    onLabelSelect: (id: string) => void
    gmailDisabled?: boolean
    outlookDisabled?: boolean
}

export function Sidebar({
    currentProvider,
    onProviderChange,
    labels,
    selectedLabelId,
    onLabelSelect,
    gmailDisabled,
    outlookDisabled
}: SidebarProps) {
    const [collapsed, _setCollapsed] = useState(false)

    return (
        <div className={cn("h-full bg-card border-r flex flex-col transition-all duration-300", collapsed ? "w-16" : "w-64")}>
            {/* Account Switcher */}
            <div className="p-4 border-b space-y-2">
                {!collapsed && <p className="text-xs font-semibold text-muted-foreground uppercase tracking-wider">Accounts</p>}

                <button
                    disabled={gmailDisabled}
                    onClick={() => onProviderChange("gmail")}
                    className={cn(
                        "w-full flex items-center gap-2 p-2 rounded-lg transition-colors text-left",
                        currentProvider === "gmail" ? "bg-accent text-accent-foreground" : "hover:bg-accent/50 text-muted-foreground",
                        gmailDisabled && "opacity-50 cursor-not-allowed"
                    )}
                >
                    <div className="w-6 h-6 rounded-full flex items-center justify-center bg-red-500 text-white font-bold text-xs">G</div>
                    {!collapsed && <span className="text-sm font-medium">Gmail</span>}
                </button>

                <button
                    disabled={outlookDisabled}
                    onClick={() => onProviderChange("outlook")}
                    className={cn(
                        "w-full flex items-center gap-2 p-2 rounded-lg transition-colors text-left",
                        currentProvider === "outlook" ? "bg-accent text-accent-foreground" : "hover:bg-accent/50 text-muted-foreground",
                        outlookDisabled && "opacity-50 cursor-not-allowed"
                    )}
                >
                    <div className="w-6 h-6 rounded-full flex items-center justify-center bg-blue-600 text-white font-bold text-xs">O</div>
                    {!collapsed && <span className="text-sm font-medium">Outlook</span>}
                </button>
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
