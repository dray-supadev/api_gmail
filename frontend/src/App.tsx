import { useState, useEffect } from 'react'
import { Sidebar } from './components/Sidebar'
import { MessageList } from './components/MessageList'
import { ThreadView } from './components/ThreadView'
import { QuotePreview } from './components/QuotePreview'
import { api } from './api'
import type { Message } from './api'

function App() {
  const [provider, setProvider] = useState<"gmail" | "outlook">("gmail")

  // Store tokens for each provider
  const [tokens, setTokens] = useState<{ gmail?: string, outlook?: string }>({})
  const [legacyToken, setLegacyToken] = useState<string>("")

  const [quoteId, setQuoteId] = useState<string | null>(null)
  const [bubbleVersion, setBubbleVersion] = useState<string | undefined>(undefined)

  const [messages, setMessages] = useState<Message[]>([])
  const [selectedThreadId, setSelectedThreadId] = useState<string | null>(null)
  const [loading, setLoading] = useState(false)

  // Derived active token: uses specific provider token if available, falls back to legacy/generic token
  const activeToken = provider === "gmail" ? (tokens.gmail || legacyToken) : (tokens.outlook || legacyToken)

  // Load config from URL
  useEffect(() => {
    const params = new URLSearchParams(window.location.search);

    const gmailToken = params.get("gmailToken") || undefined;
    const outlookToken = params.get("outlookToken") || undefined;
    const t = params.get("token") || "";

    setTokens({ gmail: gmailToken, outlook: outlookToken });
    setLegacyToken(t);

    const p = params.get("provider");
    if (p === "gmail" || p === "outlook") setProvider(p);

    setQuoteId(params.get("quoteId"));
    setBubbleVersion(params.get("bubbleVersion") || undefined);
  }, [])

  // Fetch messages when token is available
  useEffect(() => {
    // Clear messages when provider switches to avoid showing wrong data
    setMessages([]);
    setSelectedThreadId(null);

    if (!activeToken) return;
    setLoading(true);
    api.listMessages(activeToken, provider)
      .then(setMessages)
      .catch(err => {
        console.error("Failed to load messages:", err);
        setMessages([]); // Ensure empty on error
      })
      .finally(() => setLoading(false));
  }, [activeToken, provider]);

  const selectedMessage = messages.find(m => m.id === selectedThreadId || m.thread_id === selectedThreadId);

  return (
    <div className="flex h-screen bg-background text-foreground overflow-hidden font-sans">
      <Sidebar currentProvider={provider} onProviderChange={setProvider} />

      {/* Messages List - Always visible */}
      {loading ? (
        <div className="w-[350px] border-r flex items-center justify-center text-muted-foreground bg-slate-50">Loading messages...</div>
      ) : (
        <MessageList
          messages={messages}
          selectedId={selectedThreadId}
          onSelect={setSelectedThreadId}
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
                - Show ThreadView.
          */}

        {quoteId && activeToken ? (
          // QUOTE MODE
          selectedThreadId && selectedMessage ? (
            <QuotePreview
              key={selectedThreadId} // Re-mount when thread changes to reset/update fields
              quoteId={quoteId}
              version={bubbleVersion}
              token={activeToken}
              provider={provider}
              initialTo={selectedMessage.from ? [selectedMessage.from.replace(/<.*>/, "").trim()] : []}
              initialSubject={selectedMessage.subject ? `RE: ${selectedMessage.subject}` : "Quote Proposal"}
              threadId={selectedMessage.thread_id}
              onClose={() => setQuoteId(null)}
              className="w-full h-full border-none shadow-none"
            />
          ) : (
            <div className="flex flex-col items-center justify-center h-full text-muted-foreground space-y-4">
              <div className="p-4 rounded-full bg-slate-100">
                <svg xmlns="http://www.w3.org/2000/svg" className="h-10 w-10 text-slate-400" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                  <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={1} d="M3 8l7.89 5.26a2 2 0 002.22 0L21 8M5 19h14a2 2 0 002-2V7a2 2 0 00-2-2H5a2 2 0 00-2 2v10a2 2 0 002 2z" />
                </svg>
              </div>
              <p className="font-medium text-lg">Select conversation to continue</p>
              <p className="text-sm max-w-xs text-center">Choose an email thread from the left to reply with a quote proposal.</p>
            </div>
          )
        ) : (
          // NORMAL READING MODE
          <ThreadView threadId={selectedThreadId} />
        )}
      </div>
    </div>
  )
}

export default App
