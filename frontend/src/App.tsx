import { useState } from 'react'
import { Sidebar } from './components/Sidebar'
import { MessageList } from './components/MessageList'
import { ThreadView } from './components/ThreadView'

function App() {
  const [provider, setProvider] = useState<"gmail" | "outlook">("gmail")
  const [selectedThreadId, setSelectedThreadId] = useState<string | null>(null)

  // Mock Data
  const messages = [
    { id: "1", from: "Klr Obi, Kir @...", subject: "EDITED reply to Quote proposal from Norfolk Terminal", snippet: "This is the latest rate request and should be at the top...", date: "Sep 11", unread: false, has_attachments: true },
    { id: "2", from: "Kir @ Co-Founder AI", subject: "Reminder for: Quote proposal from Norfolk Terminal", snippet: "Just checking in on this...", date: "Sep 04", unread: true, has_attachments: false },
    { id: "3", from: "Klr Obi, Jonathan Sinton", subject: "This is the latest rate request and should be at the top", snippet: "Please review the attached documents...", date: "Sep 04", unread: false, has_attachments: false },
  ]

  return (
    <div className="flex h-screen bg-background text-foreground overflow-hidden font-sans">
      <Sidebar currentProvider={provider} onProviderChange={setProvider} />
      <MessageList
        messages={messages}
        selectedId={selectedThreadId}
        onSelect={setSelectedThreadId}
      />
      <ThreadView threadId={selectedThreadId} />
    </div>
  )
}

export default App
