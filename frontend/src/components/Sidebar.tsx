import { useState } from "react"
import { Inbox, ChevronDown } from "lucide-react"
import { cn } from "@/lib/utils"

interface SidebarProps {
    currentProvider: "gmail" | "outlook"
    onProviderChange: (provider: "gmail" | "outlook") => void
}

export function Sidebar({ currentProvider, onProviderChange }: SidebarProps) {
    const [collapsed, _setCollapsed] = useState(false)

    return (
        <div className={cn("h-screen bg-card border-r flex flex-col transition-all duration-300", collapsed ? "w-16" : "w-64")}>
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

            {/* Menu - Only Inbox for now */}
            <nav className="flex-1 p-2 space-y-1">
                <div
                    className={cn(
                        "flex items-center gap-3 px-3 py-2 rounded-md cursor-pointer text-sm font-medium transition-colors bg-accent text-accent-foreground"
                    )}
                >
                    <Inbox className="w-5 h-5 shrink-0" />
                    {!collapsed && (
                        <span className="flex-1">Inbox</span>
                    )}
                </div>
            </nav>
        </div>
    )
}
