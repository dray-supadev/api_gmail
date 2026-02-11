import { useState } from "react"
import { Inbox, Folder, Send, FileText, Trash2, Mail, Archive, Plus, Lock, Globe } from "lucide-react"
import { cn } from "@/lib/utils"
import type { Label, UserProfile } from "../api"
import { ConnectPopup } from "./ConnectPopup"

interface SidebarProps {
    currentProvider: "gmail" | "outlook" | "postmark"
    onProviderChange: (provider: "gmail" | "outlook" | "postmark") => void
    labels: Label[]
    userProfile?: UserProfile | null
    selectedLabelId: string
    onLabelSelect: (id: string) => void
    gmailDisabled?: boolean
    outlookDisabled?: boolean
    onNewMail: () => void
    bubbleVersion: string
}

export function Sidebar({
    currentProvider,
    onProviderChange,
    labels,
    userProfile,
    selectedLabelId,
    onLabelSelect,
    gmailDisabled,
    outlookDisabled,
    onNewMail,
    bubbleVersion
}: SidebarProps) {
    const [collapsed, _setCollapsed] = useState(false)
    const [connectPopup, setConnectPopup] = useState<{ isOpen: boolean, provider: string } | null>(null)

    const handleProviderClick = (provider: "gmail" | "outlook") => {
        const isDisabled = provider === "gmail" ? gmailDisabled : outlookDisabled;
        if (isDisabled) {
            setConnectPopup({ isOpen: true, provider: provider === "gmail" ? "Gmail" : "Outlook" });
        } else {
            onProviderChange(provider);
        }
    }

    return (
        <div className={cn("h-full bg-card border-r flex flex-col transition-all duration-300", collapsed ? "w-16" : "w-64")}>
            <ConnectPopup
                isOpen={!!connectPopup?.isOpen}
                onClose={() => setConnectPopup(null)}
                providerName={connectPopup?.provider || ""}
                version={bubbleVersion}
            />

            {/* New Mail Button */}
            <div className="p-4 pb-2">
                <button
                    onClick={onNewMail}
                    className={cn(
                        "w-full flex items-center gap-2 px-4 py-3 rounded-lg bg-black text-white hover:bg-gray-800 transition-colors shadow-sm",
                        collapsed ? "justify-center px-0" : ""
                    )}
                >
                    <Plus className="w-5 h-5" />
                    {!collapsed && <span className="font-semibold">New Mail</span>}
                </button>
            </div>

            {/* Account Switcher */}
            <div className="p-4 border-b space-y-2">
                {!collapsed && <p className="text-xs font-semibold text-muted-foreground uppercase tracking-wider">Accounts</p>}

                <button
                    onClick={() => handleProviderClick("gmail")}
                    className={cn(
                        "w-full flex items-center justify-between p-2 rounded-lg transition-colors text-left group",
                        currentProvider === "gmail" ? "bg-accent text-accent-foreground" : "hover:bg-accent/50 text-muted-foreground",
                    )}
                >
                    <div className="flex items-center gap-2">
                        <div className="w-6 h-6 rounded-full flex items-center justify-center bg-red-500 text-white font-bold text-xs">G</div>
                        {!collapsed && <span className="text-sm font-medium">Gmail</span>}
                    </div>
                    {gmailDisabled && <Lock className="w-3 h-3 text-muted-foreground opacity-50" />}
                </button>

                <button
                    onClick={() => handleProviderClick("outlook")}
                    className={cn(
                        "w-full flex items-center justify-between p-2 rounded-lg transition-colors text-left group",
                        currentProvider === "outlook" ? "bg-accent text-accent-foreground" : "hover:bg-accent/50 text-muted-foreground",
                    )}
                >
                    <div className="flex items-center gap-2">
                        <div className="w-6 h-6 rounded-full flex items-center justify-center bg-blue-600 text-white font-bold text-xs">O</div>
                        {!collapsed && <span className="text-sm font-medium">Outlook</span>}
                    </div>
                    {outlookDisabled && <Lock className="w-3 h-3 text-muted-foreground opacity-50" />}
                </button>

                <button
                    onClick={() => onProviderChange("postmark")}
                    className={cn(
                        "w-full flex items-center gap-2 p-2 rounded-lg transition-colors text-left",
                        currentProvider === "postmark" ? "bg-accent text-accent-foreground" : "hover:bg-accent/50 text-muted-foreground",
                    )}
                >
                    <div className="w-6 h-6 rounded-full flex items-center justify-center bg-purple-600 text-white font-bold text-xs">
                        <Globe className="w-3 h-3" />
                    </div>
                    {!collapsed && <span className="text-sm font-medium">Custom Domain</span>}
                </button>
            </div>

            {/* Profile Info */}
            {
                userProfile && !collapsed && (
                    <div className="px-4 py-2 border-b">
                        <p className="text-xs font-semibold text-muted-foreground uppercase tracking-wider mb-1">Authenticated as</p>
                        <div className="flex items-center gap-2 overflow-hidden">
                            {userProfile.picture ? (
                                <img src={userProfile.picture} alt="Profile" className="w-5 h-5 rounded-full" />
                            ) : (
                                <div className="w-5 h-5 rounded-full bg-slate-200 flex items-center justify-center text-[10px] font-bold text-slate-500">
                                    {(userProfile.email || "U").charAt(0).toUpperCase()}
                                </div>
                            )}
                            <span className="text-sm truncate text-foreground" title={userProfile.email}>
                                {userProfile.email}
                            </span>
                        </div>
                    </div>
                )
            }

            {/* Menu - Labels */}
            {currentProvider === "postmark" ? (
                <div className="flex-1 p-4 text-center text-muted-foreground text-sm mt-10">
                    <p>Sending Only Mode</p>
                    <p className="text-xs opacity-70 mt-1">Inbox not available for custom domains.</p>
                </div>
            ) : (
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
                            <button
                                key={label.id}
                                onClick={() => onLabelSelect(label.id)}
                                className={cn(
                                    "w-full flex items-center gap-2 p-2 rounded-lg text-sm transition-colors text-left",
                                    selectedLabelId === label.id ? "bg-secondary text-secondary-foreground font-medium" : "text-muted-foreground hover:bg-secondary/50 hover:text-foreground"
                                )}
                            >
                                <Icon className="w-4 h-4" />
                                {!collapsed && <span className="truncate">{label.name}</span>}
                            </button>
                        )
                    })}
                </nav>
            )}
        </div>
    )
}
onClick = {() => onLabelSelect(label.id)}
className = {
    cn(
                                "flex items-center gap-3 px-3 py-2 rounded-md cursor-pointer text-sm font-medium transition-colors hover:bg-accent/50",
        selectedLabelId === label.id ? "bg-accent text-accent-foreground" : "text-muted-foreground"
                            )}
                        >
    <Icon className="w-5 h-5 shrink-0" />
{ !collapsed && <span className="flex-1 truncate">{label.name}</span> }
                        </div >
                    );
                })}

{
    labels.length === 0 && !collapsed && (
        <div className="px-3 py-4 text-xs text-center text-muted-foreground italic">
            No labels found
        </div>
    )
}
            </nav >
        </div >
    )
}
