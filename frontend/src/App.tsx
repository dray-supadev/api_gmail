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
    <div className="flex h-screen bg-background text-foreground overflow-hidden font-sans relative">
      <Sidebar currentProvider={provider} onProviderChange={setProvider} />

      {/* Messages */}
      {loading ? (
        <div className="w-[400px] border-r flex items-center justify-center text-muted-foreground">Loading messages...</div>
      ) : (
        <MessageList
          messages={messages}
          selectedId={selectedThreadId}
          onSelect={setSelectedThreadId}
        />
      )}

      {/* Main View */}
      <div className="flex-1 flex overflow-hidden relative">
        <div className="flex-1 min-w-0">
          <ThreadView threadId={selectedThreadId} />
        </div>
      </div>

      {/* Quote Preview Modal Overlay */}
      {quoteId && activeToken && (
        <div className="absolute inset-0 z-50 bg-background/80 backdrop-blur-sm flex items-center justify-center p-4 sm:p-8">
          <div className="w-full h-full max-w-6xl shadow-2xl rounded-lg overflow-hidden border ring-1 ring-border">
            <QuotePreview
              quoteId={quoteId}
              version={bubbleVersion}
              token={activeToken}
              provider={provider}
              initialTo={selectedMessage?.from ? [selectedMessage.from.replace(/<.*>/, "").trim()] : []}
              initialSubject={selectedMessage?.subject ? `RE: ${selectedMessage.subject}` : "Quote Proposal"}
              threadId={selectedMessage?.thread_id}
              onClose={() => setQuoteId(null)}
              className="w-full h-full border-none shadow-none"
            />
          </div>
        </div>
      )}
    </div>
  )
}

export default App
