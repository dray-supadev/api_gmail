export interface Message {
    id: string;
    thread_id: string;
    snippet: string;
    subject?: string;
    from?: string;
    to?: string[];
    cc?: string[];
    date?: string;
    unread: boolean;
    has_attachments: boolean;
}

export interface UserProfile {
    email: string;
    name?: string;
    picture?: string;
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
    cc?: string[];
    subject: string;
    thread_id?: string;
    comment?: string;
    pdf_export_settings?: string[];
    html_body?: string;
    pdf_base64?: string;
    pdf_name?: string;
    maildata_identificator?: string;
    company?: string;
}

const API_BASE = import.meta.env.PROD ? "" : "http://localhost:3000";

let globalApiKey: string | null = null;

async function handleResponse(res: Response) {
    if (!res.ok) {
        let errorMessage = `Error: ${res.status}`;
        try {
            const errorData = await res.json();
            errorMessage = errorData.error || errorData.details || errorMessage;
        } catch (e) {
            // fallback to status text
        }
        throw new Error(errorMessage);
    }
    return res.json();
}

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
        const data = await handleResponse(res);
        return data.messages || [];
    },

    async listLabels(token: string, provider: string): Promise<Label[]> {
        const res = await fetch(`${API_BASE}/api/labels?provider=${provider}`, {
            headers: {
                "Authorization": `Bearer ${token}`,
                ...(globalApiKey ? { "x-api-key": globalApiKey } : {})
            }
        });
        return await handleResponse(res);
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
        return await handleResponse(res);
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
        return await handleResponse(res);
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
        return await handleResponse(res);
    },

    async getProfile(token: string, provider: string, company?: string): Promise<UserProfile> {
        let url = `${API_BASE}/api/profile?provider=${provider}`;
        if (company) {
            url += `&company=${encodeURIComponent(company)}`;
        }

        const res = await fetch(url, {
            headers: {
                "Authorization": `Bearer ${token}`,
                ...(globalApiKey ? { "x-api-key": globalApiKey } : {})
            }
        });
        return await handleResponse(res);
    }
};
