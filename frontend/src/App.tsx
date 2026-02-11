import { useState, useEffect, useCallback } from 'react'
import { Sidebar } from './components/Sidebar'
import { MessageList } from './components/MessageList'
import { QuotePreview } from './components/QuotePreview'
import { api } from './api'
import type { Message, Label, UserProfile } from './api'
import { X } from 'lucide-react'

function App() {
  const [provider, setProvider] = useState<"gmail" | "outlook" | "postmark">("gmail")

  // Store tokens for each provider
  const [tokens, setTokens] = useState<{ gmail?: string, outlook?: string }>({})
  const [legacyToken, setLegacyToken] = useState<string>("")
  const [company, setCompany] = useState<string | null>(null)

  const [quoteId, setQuoteId] = useState<string | null>(null)
  const [bubbleVersion, setBubbleVersion] = useState<string | undefined>(undefined)
  const [pdfExportSettings, setPdfExportSettings] = useState<string[]>([])

  // Data passed from Bubble to avoid API calls (for PDF)
  const [pdfBase64, setPdfBase64] = useState<string | null>(null)
  const [pdfName, setPdfName] = useState<string | null>(null)

  // ... inside useEffect


  const [messages, setMessages] = useState<Message[]>([])
  const [labels, setLabels] = useState<Label[]>([])
  const [userProfile, setUserProfile] = useState<UserProfile | null>(null)
  const [selectedLabelId, setSelectedLabelId] = useState<string>("INBOX")
  const [selectedThreadId, setSelectedThreadId] = useState<string | null>(null)
  const [loading, setLoading] = useState(false)
  const [searchQuery, setSearchQuery] = useState("")

  // Error state for session expiry
  const [authError, setAuthError] = useState(false)

  // Derived active token: uses specific provider token if available, falls back to legacy/generic token
  const activeToken = provider === "gmail" ? (tokens.gmail || legacyToken) : (tokens.outlook || legacyToken)

  const [isConfigLoaded, setIsConfigLoaded] = useState(false)

  // Load config from URL
  useEffect(() => {
    const params = new URLSearchParams(window.location.search);

    const apiKey = params.get("apiKey");
    if (apiKey) {
      api.setApiKey(apiKey);
    } else {
      // Fallback to check if we have the injected key
      console.log("No API Key in URL, checking for injected key...");
    }

    const gmailToken = params.get("gmailToken") || undefined;
    const outlookToken = params.get("outlookToken") || undefined;
    const t = params.get("token") || "";

    setTokens({ gmail: gmailToken, outlook: outlookToken });
    setLegacyToken(t);

    const p = params.get("provider");
    if (p === "gmail" || p === "outlook") setProvider(p);

    setQuoteId(params.get("quoteId"));
    setBubbleVersion(params.get("bubbleVersion") || undefined);

    const settingsParam = params.get("pdfExportSettings");
    if (settingsParam) {
      setPdfExportSettings(settingsParam.split(",").filter(Boolean));
    }

    setPdfBase64(params.get("pdfBase64"));
    setPdfName(params.get("pdfName"));
    setCompany(params.get("company"));

    setIsConfigLoaded(true);
  }, [])

  // Listen for config messages (for large data)
  useEffect(() => {
    const handleMessage = (event: MessageEvent) => {
      if (event.data && event.data.type === 'GMAIL_WIDGET_CONFIG') {
        const config = event.data.config;
        console.log("Received config via postMessage:", { ...config, gmailToken: '***', outlookToken: '***' });

        if (config.apiKey) api.setApiKey(config.apiKey);
        if (config.gmailToken || config.outlookToken) {
          setTokens(prev => ({ ...prev, gmail: config.gmailToken, outlook: config.outlookToken }));
        }
        if (config.token) setLegacyToken(config.token);
        if (config.provider) setProvider(config.provider);
        if (config.quoteId) setQuoteId(config.quoteId);
        if (config.bubbleVersion) setBubbleVersion(config.bubbleVersion);
        if (config.pdfExportSettings) {
          setPdfExportSettings(Array.isArray(config.pdfExportSettings) ? config.pdfExportSettings : config.pdfExportSettings.split(",").filter(Boolean));
        }

        // Large data fields
        if (config.pdfBase64) setPdfBase64(config.pdfBase64);
        if (config.pdfName) setPdfName(config.pdfName);
      }
    };

    window.addEventListener('message', handleMessage);
    return () => window.removeEventListener('message', handleMessage);
  }, []);

  // Reset label and search when provider changes
  useEffect(() => {
    setSelectedLabelId("INBOX");
    setSearchQuery("");
  }, [provider]);

  // Fetch labels when token is available
  useEffect(() => {
    if (!isConfigLoaded || !activeToken) return;
    setLoading(true);
    api.listLabels(activeToken, provider)
      .then(setLabels)
      .catch(err => console.error("Failed to load labels:", err));

    api.getProfile(activeToken, provider)
      .then(setUserProfile)
      .catch(err => console.error("Failed to load profile:", err));
  }, [activeToken, provider, isConfigLoaded]);

  // Fetch messages when token or label changes
  useEffect(() => {
    // Clear messages when provider or label switches
    setMessages([]);
    setSelectedThreadId(null);
    setAuthError(false);

    if (!isConfigLoaded || !activeToken) return;
    setLoading(true);

    api.listMessages(activeToken, provider, {
      label_ids: selectedLabelId,
      q: searchQuery || undefined
    })
      .then(setMessages)
      .catch(err => {
        console.error("Failed to load messages:", err);
        setMessages([]);
        if (err.message.includes("401") || err.message === "Unauthorized") {
          setAuthError(true);
        }
      })
      .finally(() => setLoading(false));
  }, [activeToken, provider, selectedLabelId, searchQuery, isConfigLoaded]);

  // Initial debug log
  useEffect(() => {
    const params = new URLSearchParams(window.location.search);
    console.log("Widget initialized with:", {
      hasGoogleToken: !!(params.get("gmailToken") || params.get("token")),
      hasOutlookToken: !!params.get("outlookToken"),
      hasApiKey: !!params.get("apiKey"),
      provider
    });
  }, []);

  // Automatically select provider if only one token is available
  useEffect(() => {
    if (tokens.gmail && !tokens.outlook) {
      setProvider("gmail");
    } else if (!tokens.gmail && tokens.outlook) {
      setProvider("outlook");
    }
    // If both are present, we respect the current selection or default
  }, [tokens.gmail, tokens.outlook]);

  const handleClose = useCallback(() => {
    window.parent.postMessage({ type: 'GMAIL_WIDGET_CLOSE' }, '*');
  }, []);

  const selectedMessage = messages.find(m => m.id === selectedThreadId || m.thread_id === selectedThreadId);

  const handleMoveMessage = useCallback(async (messageId: string, newLabelId: string) => {
    if (!activeToken) return;

    try {
      // Optimistic update: Remove message from list immediately
      setMessages(prev => prev.filter(m => m.id !== messageId));
      if (selectedThreadId === messageId) {
        setSelectedThreadId(null);
        // Do NOT clear quoteId, otherwise we lose the widget context
      }

      const req: any = {
        ids: [messageId],
        add_label_ids: [newLabelId]
      };

      if (provider === "gmail") {
        // For Gmail, we typically remove the current label (e.g. INBOX) when moving
        // If we are in "All Mail" or search, we might not want to remove anything, but assuming usage from a folder:
        if (selectedLabelId && selectedLabelId !== "TRASH" && selectedLabelId !== "SPAM") {
          if (selectedLabelId !== newLabelId) {
            req.remove_label_ids = [selectedLabelId];
          }
        }
      }

      await api.modifyLabels(activeToken, provider, req);

    } catch (err) {
      console.error("Failed to move message:", err);
      // Revert optimistic update (simplified: just reload messages)
      // trigger reload...
    }
  }, [activeToken, provider, selectedLabelId, selectedThreadId]);

  const [isNewMailMode, setIsNewMailMode] = useState(false);

  const startNewMail = useCallback(() => {
    setSelectedThreadId(null);
    setIsNewMailMode(true);
  }, []);

  const handleProviderChange = (newProvider: "gmail" | "outlook" | "postmark") => {
    setProvider(newProvider);
    setIsNewMailMode(false);
    setSelectedThreadId(null);
  };

  const handleArchive = useCallback(async (messageId: string) => {
    // Archive = Remove INBOX label (Gmail) or Move to Archive (Outlook)
    if (provider === "gmail") {
      // Gmail: Remove "INBOX" label
      // Assuming "INBOX" is the ID for the inbox.
      // If we are currently in INBOX label, we remove it.
      // If we are in another label, archiving usually implies removing INBOX if present.
      // But our API `batchModify` takes `removeLabelIds`.
      // We'll optimistically remove from list.
      setMessages(prev => prev.filter(m => m.id !== messageId));
      if (selectedThreadId === messageId) setSelectedThreadId(null);

      try {
        await api.modifyLabels(activeToken!, provider, {
          ids: [messageId],
          remove_label_ids: ["INBOX"]
        });
      } catch (e) {
        console.error("Archive failed", e);
        // Handle revert if needed
      }
    } else {
      // Outlook: Move to "Archive" folder.
      // We need to find the Archive folder ID.
      // We can fallback to just removing from current list if we can't find it, 
      // but for Outlook we really need to Move.
      const archiveLabel = labels.find(l => l.name.toLowerCase() === "archive");
      if (archiveLabel) {
        handleMoveMessage(messageId, archiveLabel.id);
      } else {
        console.warn("Archive folder not found for Outlook");
        alert("Archive folder not found.");
      }
    }
  }, [activeToken, provider, labels, handleMoveMessage, selectedThreadId]);

  const handleDelete = useCallback(async (messageId: string) => {
    setMessages(prev => prev.filter(m => m.id !== messageId));
    if (selectedThreadId === messageId) setSelectedThreadId(null);

    // Gmail: Add TRASH label.
    // Outlook: Move to Deleted Items.

    try {
      if (provider === "gmail") {
        await api.modifyLabels(activeToken!, provider, {
          ids: [messageId],
          add_label_ids: ["TRASH"]
        });
      } else {
        // Outlook: Find TRASH folder
        const trashLabel = labels.find(l => l.name.toLowerCase().includes("deleted") || l.name.toLowerCase().includes("trash") || l.id === "TRASH");
        if (trashLabel) {
          handleMoveMessage(messageId, trashLabel.id);
        } else {
          console.warn("Trash folder not found for Outlook");
        }
      }
    } catch (e) {
      console.error("Delete failed", e);
    }
  }, [activeToken, provider, labels, handleMoveMessage, selectedThreadId]);

  if (isConfigLoaded && !tokens.gmail && !tokens.outlook && !legacyToken) {
    return (
      <div className="fixed inset-0 flex items-center justify-center bg-background text-foreground">
        <div className="text-center space-y-4 p-8 max-w-md">
          <div className="mx-auto w-16 h-16 bg-slate-100 rounded-full flex items-center justify-center">
            <svg xmlns="http://www.w3.org/2000/svg" className="h-8 w-8 text-slate-400" fill="none" viewBox="0 0 24 24" stroke="currentColor">
              <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M12 9v2m0 4h.01m-6.938 4h13.856c1.54 0 2.502-1.667 1.732-3L13.732 4c-.77-1.333-2.694-1.333-3.464 0L3.34 16c-.77 1.333.192 3 1.732 3z" />
            </svg>
          </div>
          <h2 className="text-2xl font-bold">No Email Account Connected</h2>
          <p className="text-muted-foreground">
            Please connect either a Gmail or Outlook account in your settings to use this widget.
          </p>
          <button
            onClick={handleClose}
            className="px-4 py-2 bg-primary text-primary-foreground rounded-md hover:bg-primary/90 transition"
          >
            Close Widget
          </button>
        </div>
      </div>
    )
  }

  return (
    <div className="fixed inset-0 flex bg-background text-foreground overflow-hidden font-sans">
      <Sidebar
        currentProvider={provider}
        onProviderChange={handleProviderChange}
        labels={labels}
        userProfile={userProfile}
        selectedLabelId={selectedLabelId}
        onLabelSelect={setSelectedLabelId}
        gmailDisabled={!tokens.gmail && !legacyToken}
        outlookDisabled={!tokens.outlook}
        onNewMail={startNewMail}
        bubbleVersion={bubbleVersion || 'version-test'}
      />

      {/* Messages List - Always visible */}
      {loading ? (
        <div className="w-[350px] border-r flex items-center justify-center text-muted-foreground bg-slate-50">Loading messages...</div>
      ) : authError ? (
        <div className="w-[350px] border-r flex flex-col items-center justify-center p-6 text-center bg-red-50 text-red-800 space-y-4">
          <div className="p-3 bg-red-100 rounded-full">
            <svg xmlns="http://www.w3.org/2000/svg" className="h-8 w-8" fill="none" viewBox="0 0 24 24" stroke="currentColor">
              <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M12 9v2m0 4h.01m-6.938 4h13.856c1.54 0 2.502-1.667 1.732-3L13.732 4c-.77-1.333-2.694-1.333-3.464 0L3.34 16c-.77 1.333.192 3 1.732 3z" />
            </svg>
          </div>
          <div>
            <h3 className="font-bold text-lg">Session Expired</h3>
            <p className="text-sm mt-2">Your access token is invalid or has expired.</p>
          </div>
          <button
            onClick={() => window.location.reload()}
            className="px-4 py-2 bg-red-600 text-white rounded-md text-sm hover:bg-red-700 transition"
          >
            Reload Page
          </button>
        </div>
      ) : (
        <MessageList
          messages={messages}
          selectedId={selectedThreadId}
          onSelect={setSelectedThreadId}
          onSearch={setSearchQuery}
          labelName={labels.find(l => l.id === selectedLabelId)?.name || "Inbox"}
          labels={labels}
          onMove={handleMoveMessage}
          onArchive={handleArchive}
          onDelete={handleDelete}
        />
      )}

      {/* Main View Area */}
      <div className="flex-1 flex flex-col overflow-hidden relative bg-white">
        {/* 
             MODE HANDLING:
             1. If quoteId is set: We are in "Quote Composer Mode".
                - If no message selected: Show "Select conversation".
                - If message selected: Show QuotePreview (Composer) pre-filled.
             2. If no quoteId: Standard "Reading Mode".
          */}

        {quoteId && activeToken ? (
          // QUOTE MODE
          (selectedThreadId && selectedMessage) || isNewMailMode ? (
            <QuotePreview
              key={selectedThreadId || 'new-mail'}
              quoteId={quoteId}
              version={bubbleVersion}
              token={activeToken}
              provider={provider}
              initialTo={isNewMailMode ? [] : (selectedMessage?.from ? [
                (() => {
                  const match = selectedMessage.from.match(/<([^>]+)>/);
                  return match ? match[1] : selectedMessage.from.trim();
                })()
              ] : [])}
              initialSubject={isNewMailMode ? "Quote Proposal" : (selectedMessage?.subject ? `RE: ${selectedMessage.subject}` : "Quote Proposal")}
              threadId={isNewMailMode ? undefined : selectedMessage?.thread_id}
              onClose={() => {
                setQuoteId(null)
                setIsNewMailMode(false)
                window.parent.postMessage({ type: 'GMAIL_WIDGET_CLOSE' }, '*')
              }}
              pdfExportSettings={pdfExportSettings}
              pdfBase64={pdfBase64 || undefined}
              pdfName={pdfName || undefined}
              className="w-full h-full border-none shadow-none"
              // Pass company if needed by QuotePreview? Not currently used there but might be useful for Postmark context
              company={company || undefined}
            />
          ) : (
            <div className="flex flex-col items-center justify-center h-full text-muted-foreground space-y-4">
              <div className="p-4 rounded-full bg-slate-100">
                <svg xmlns="http://www.w3.org/2000/svg" className="h-10 w-10 text-slate-400" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                  <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={1} d="M3 8l7.89 5.26a2 2 0 002.22 0L21 8M5 19h14a2 2 0 002-2V7a2 2 0 00-2-2H5a2 2 0 00-2 2v10a2 2 0 002 2z" />
                </svg>
              </div>
              <p className="font-medium text-lg">Select conversation to continue</p>
              <p className="text-sm max-w-xs text-center">Choose an email thread from the left or click "New Mail" to send a proposal.</p>
            </div>
          )
        ) : (
          // NO QUOTE ID or NO TOKEN (Should be caught by early return above, but safe fallback)
          <div className="flex flex-col items-center justify-center h-full text-muted-foreground space-y-4">
            <div className="p-4 rounded-full bg-slate-100">
              <svg xmlns="http://www.w3.org/2000/svg" className="h-10 w-10 text-slate-400" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={1} d="M12 15v2m-6 4h12a2 2 0 002-2v-6a2 2 0 00-2-2H6a2 2 0 00-2 2v6a2 2 0 002 2zm10-10V7a4 4 0 00-8 0v4h8z" />
              </svg>
            </div>
            <p className="font-medium text-lg">Waiting for configuration...</p>
            <p className="text-sm max-w-xs text-center">Please ensure the widget is opened from a valid Quote page in Bubble.</p>
          </div>
        )}

        {/* Global Close Button (Always visible) */}
        <button
          onClick={handleClose}
          className="fixed top-5 right-5 p-2 rounded-full bg-white hover:bg-slate-100 text-slate-400 hover:text-slate-600 transition-all z-[99999] shadow-lg border border-slate-200"
          title="Закрыть"
        >
          <X className="w-6 h-6" />
        </button>
      </div>
    </div>
  )
}

export default App
