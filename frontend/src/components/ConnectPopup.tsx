import { X } from "lucide-react"

interface ConnectPopupProps {
    isOpen: boolean
    onClose: () => void
    providerName: string
    version: string
}

export function ConnectPopup({ isOpen, onClose, providerName, version }: ConnectPopupProps) {
    if (!isOpen) return null

    // Extract version prefix if needed, or use as is. 
    // Usually version is like "version-test" or "version-live"
    // URL format: https://app.drayinsight.com/<version>/app?tab=Integrations&v=settings

    const connectUrl = `https://app.drayinsight.com/${version}/app?tab=Integrations&v=settings`

    return (
        <div className="fixed inset-0 z-50 flex items-center justify-center bg-black/50 backdrop-blur-sm">
            <div className="bg-white rounded-lg shadow-xl p-6 w-full max-w-sm relative">
                <button
                    onClick={onClose}
                    className="absolute top-4 right-4 text-gray-500 hover:text-gray-700"
                >
                    <X className="w-5 h-5" />
                </button>

                <div className="text-center space-y-4">
                    <div className="w-12 h-12 bg-blue-100 rounded-full flex items-center justify-center mx-auto">
                        <svg xmlns="http://www.w3.org/2000/svg" className="h-6 w-6 text-blue-600" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                            <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M13.828 10.172a4 4 0 00-5.656 0l-4 4a4 4 0 105.656 5.656l1.102-1.101m-.758-4.899a4 4 0 005.656 0l4-4a4 4 0 00-5.656-5.656l-1.1 1.1" />
                        </svg>
                    </div>

                    <h3 className="text-lg font-semibold">Connect {providerName}</h3>

                    <p className="text-sm text-gray-600">
                        To use {providerName}, you need to connect your account in the integration settings.
                    </p>

                    <a
                        href={connectUrl}
                        target="_blank"
                        rel="noopener noreferrer"
                        className="block w-full py-2 px-4 bg-primary text-primary-foreground hover:bg-primary/90 rounded-md transition-colors font-medium bg-black text-white"
                        onClick={onClose} // Optional: close on click
                    >
                        Go to Settings
                    </a>
                </div>
            </div>
        </div>
    )
}
