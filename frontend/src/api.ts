export interface Message {
    id: string;
    thread_id: string;
    snippet: string;
    subject?: string;
    from?: string;
    date?: string;
    unread: boolean;
    has_attachments: boolean;
}

export interface QuotePreviewParams {
    quote_id: string;
    version?: string;
    comment?: string;
}

export interface SendQuoteRequest {
    quote_id: string;
    version?: string;
    provider: string;
    to: string[];
    subject: string;
    thread_id?: string;
    comment?: string;
}

const API_BASE = import.meta.env.PROD ? "" : "http://localhost:3000";

export const api = {
    async listMessages(token: string, provider: string): Promise<Message[]> {
        const res = await fetch(`${API_BASE}/api/messages?provider=${provider}`, {
            headers: {
                "Authorization": `Bearer ${token}`
            }
        });
        if (res.status === 401) throw new Error("Unauthorized");
        if (!res.ok) throw new Error("Failed to fetch messages");
        const data = await res.json();
        return data.messages || [];
    },

    async getThread(token: string, provider: string, threadId: string) {
        const res = await fetch(`${API_BASE}/api/threads/${threadId}?provider=${provider}`, {
            headers: {
                "Authorization": `Bearer ${token}`
            }
        });
        if (!res.ok) throw new Error("Failed to fetch thread");
        return await res.json();
    },

    async previewQuote(params: QuotePreviewParams) {
        const query = new URLSearchParams({
            quote_id: params.quote_id,
            ...(params.version ? { version: params.version } : {}),
            ...(params.comment ? { comment: params.comment } : {})
        });
        const res = await fetch(`${API_BASE}/api/quote/preview?${query}`);
        if (!res.ok) {
            const text = await res.text();
            throw new Error(`Failed to preview: ${res.status} ${text}`);
        }
        return await res.json();
    },

    async sendQuote(token: string, req: SendQuoteRequest) {
        const res = await fetch(`${API_BASE}/api/quote/send`, {
            method: "POST",
            headers: {
                "Content-Type": "application/json",
                "Authorization": `Bearer ${token}`
            },
            body: JSON.stringify(req)
        });
        if (!res.ok) throw new Error("Failed to send quote");
        return await res.json();
    }
};
