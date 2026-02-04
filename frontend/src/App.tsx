import { useState, useEffect } from 'react'
import { Sidebar } from './components/Sidebar'
import { MessageList } from './components/MessageList'
import { ThreadView } from './components/ThreadView'
import { QuotePreview } from './components/QuotePreview'
import { api } from './api'
import type { Message } from './api'

function App() {
  const [provider, setProvider] = useState<"gmail" | "outlook">("gmail")
  const [token, setToken] = useState<string>("")
  const [quoteId, setQuoteId] = useState<string | null>(null)
  const [bubbleVersion, setBubbleVersion] = useState<string | undefined>(undefined)

  const [messages, setMessages] = useState<Message[]>([])
  const [selectedThreadId, setSelectedThreadId] = useState<string | null>(null)
  const [loading, setLoading] = useState(false)

  // Load config from URL
  useEffect(() => {
    const params = new URLSearchParams(window.location.search);
    const t = params.get("token");
    if (t) setToken(t);

    const p = params.get("provider");
    if (p === "gmail" || p === "outlook") setProvider(p);

    setQuoteId(params.get("quoteId"));
    setBubbleVersion(params.get("bubbleVersion") || undefined);
  }, [])

  // Fetch messages when token is available
  useEffect(() => {
    if (!token) return;
    setLoading(true);
    api.listMessages(token, provider)
      .then(setMessages)
      .catch(err => console.error("Failed to load messages:", err))
      .finally(() => setLoading(false));
  }, [token, provider]);

  const selectedMessage = messages.find(m => m.id === selectedThreadId || m.thread_id === selectedThreadId);

  return (
    <div className="flex h-screen bg-background text-foreground overflow-hidden font-sans">
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

      {/* Main View + Quote Preview */}
      <div className="flex-1 flex overflow-hidden relative">
        <div className="flex-1 min-w-0">
          <ThreadView threadId={selectedThreadId} />
        </div>

        {/* Quote Preview Panel */}
        {quoteId && token && (
          <div className="shrink-0 h-full border-l shadow-2xl z-20">
            <QuotePreview
              quoteId={quoteId}
              version={bubbleVersion}
              token={token}
              provider={provider}
              initialTo={selectedMessage?.from ? [selectedMessage.from.replace(/<.*>/, "").trim()] : []}
              initialSubject={selectedMessage?.subject ? `RE: ${selectedMessage.subject}` : "Quote Proposal"}
              threadId={selectedMessage?.thread_id}
              onClose={() => setQuoteId(null)}
            />
          </div>
        )}
      </div>
    </div>
  )
}

export default App
