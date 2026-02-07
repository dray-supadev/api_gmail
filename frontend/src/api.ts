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

export interface Label {
    id: string;
    name: string;
    label_type?: string;
}

export interface BatchModifyRequest {
    ids: string[];
    add_label_ids?: string[];
    remove_label_ids?: string[];
}

export interface QuotePreviewParams {
    quote_id: string;
    version?: string;
    comment?: string;
    pdf_export_settings?: string[];
}

export interface SendQuoteRequest {
    quote_id: string;
    version?: string;
    provider: string;
    to: string[];
    subject: string;
    thread_id?: string;
    comment?: string;
    pdf_export_settings?: string[];
    html_body?: string;
}

const API_BASE = import.meta.env.PROD ? "" : "http://localhost:3000";

let globalApiKey: string | null = null;

export const api = {
    setApiKey(key: string) {
        globalApiKey = key;
    },

    async listMessages(token: string, provider: string, params?: { label_ids?: string, q?: string, max_results?: number }): Promise<Message[]> {
        let url = `${API_BASE}/api/messages?provider=${provider}`;
        if (params?.label_ids) url += `&label_ids=${params.label_ids}`;
        if (params?.q) url += `&q=${encodeURIComponent(params.q)}`;
        if (params?.max_results) url += `&max_results=${params.max_results}`;

        const res = await fetch(url, {
            headers: {
                "Authorization": `Bearer ${token}`,
                ...(globalApiKey ? { "x-api-key": globalApiKey } : {})
            }
        });
        if (res.status === 401) throw new Error("Unauthorized");
        if (!res.ok) throw new Error("Failed to fetch messages");
        const data = await res.json();
        return data.messages || [];
    },

    async listLabels(token: string, provider: string): Promise<Label[]> {
        const res = await fetch(`${API_BASE}/api/labels?provider=${provider}`, {
            headers: {
                "Authorization": `Bearer ${token}`,
                ...(globalApiKey ? { "x-api-key": globalApiKey } : {})
            }
        });
        if (!res.ok) throw new Error("Failed to fetch labels");
        return await res.json();
    },

    async modifyLabels(token: string, provider: string, req: BatchModifyRequest) {
        const res = await fetch(`${API_BASE}/api/labels/batch-modify?provider=${provider}`, {
            method: "POST",
            headers: {
                "Content-Type": "application/json",
                "Authorization": `Bearer ${token}`,
                ...(globalApiKey ? { "x-api-key": globalApiKey } : {})
            },
            body: JSON.stringify(req)
        });
        if (!res.ok) throw new Error("Failed to modify labels");
        return await res.json();
    },

    async previewQuote(params: QuotePreviewParams) {
        const res = await fetch(`${API_BASE}/api/quote/preview`, {
            method: "POST",
            headers: {
                "Content-Type": "application/json",
                ...(globalApiKey ? { "x-api-key": globalApiKey } : {})
            },
            body: JSON.stringify(params)
        });

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
                "Authorization": `Bearer ${token}`,
                ...(globalApiKey ? { "x-api-key": globalApiKey } : {})
            },
            body: JSON.stringify(req)
        });
        if (!res.ok) throw new Error("Failed to send quote");
        return await res.json();
    }
};
