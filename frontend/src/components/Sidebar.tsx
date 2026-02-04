import { useState } from "react"
import { Inbox, Send, Archive, Trash2, Settings, ChevronDown, Plus } from "lucide-react"
import { cn } from "@/lib/utils"

interface SidebarProps {
    currentProvider: "gmail" | "outlook"
    onProviderChange: (provider: "gmail" | "outlook") => void
}

export function Sidebar({ currentProvider, onProviderChange }: SidebarProps) {
    const [collapsed, _setCollapsed] = useState(false)

    const menuItems = [
        { icon: Inbox, label: "Inbox", count: 12 },
        { icon: Send, label: "Sent", count: 0 },
        { icon: Archive, label: "Archive", count: 0 },
        { icon: Trash2, label: "Trash", count: 0 },
    ]

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
                            <p className="text-xs text-muted-foreground truncate">user@example.com</p>
                        </div>
                    )}
                    {!collapsed && <ChevronDown className="w-4 h-4 text-muted-foreground" />}
                </div>
            </div>

            {/* Compose Button */}
            <div className="p-4">
                <button className={cn("w-full bg-primary text-primary-foreground hover:bg-primary/90 h-10 rounded-md flex items-center justify-center gap-2 transition-all", collapsed ? "aspect-square p-0" : "px-4")}>
                    <Plus className="w-5 h-5" />
                    {!collapsed && <span className="font-medium">New Email</span>}
                </button>
            </div>

            {/* Menu */}
            <nav className="flex-1 p-2 space-y-1">
                {menuItems.map((item) => (
                    <div
                        key={item.label}
                        className={cn(
                            "flex items-center gap-3 px-3 py-2 rounded-md cursor-pointer text-sm font-medium transition-colors",
                            item.label === "Inbox" ? "bg-accent text-accent-foreground" : "text-muted-foreground hover:bg-accent/50 hover:text-foreground"
                        )}
                    >
                        <item.icon className="w-5 h-5 shrink-0" />
                        {!collapsed && (
                            <>
                                <span className="flex-1">{item.label}</span>
                                {item.count > 0 && <span className="bg-primary/10 text-primary text-xs px-2 py-0.5 rounded-full">{item.count}</span>}
                            </>
                        )}
                    </div>
                ))}
            </nav>

            {/* Footer / Settings */}
            <div className="p-4 border-t">
                <div className={cn("flex items-center gap-3 text-muted-foreground hover:text-foreground cursor-pointer p-2 rounded-md hover:bg-accent/50", collapsed && "justify-center")}>
                    <Settings className="w-5 h-5" />
                    {!collapsed && <span className="text-sm font-medium">Settings</span>}
                </div>
            </div>
        </div>
    )
}
