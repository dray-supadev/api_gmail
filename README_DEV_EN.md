# Gmail & Outlook API Proxy: Full Developer Guide

This service is a high-performance Rust proxy designed to integrate Bubble.io with Gmail and Outlook APIs. It resolves issues with complex MIME structure parsing, attachment handling, and provides a unified interface for different email providers.

---

## 🛠 Technology Stack
- **Language:** Rust (Edition 2021)
- **Framework:** Axum (based on Tokio)
- **HTTP Client:** Reqwest
- **MIME Parsing:** mail-parser
- **Infrastructure:** Docker (multi-stage build)

---

## 🔐 Security and Authorization

### Two-Level Authentication
1.  **API Key (x-api-key):** Protects the proxy itself.
    -   `APP_SECRET_KEY`: Full access (Admin). Used in Bubble API Connector for sending emails.
    -   `WIDGET_API_KEY`: Restricted access (Read-only/Preview). Embedded in the public JS widget.
2.  **Bearer Token (Authorization):** User's OAuth2 token (Google or Microsoft) obtained in Bubble.

### CORS
Access is restricted via the `ALLOWED_ORIGINS` variable. Always specify your Bubble application's domain.

---

## 🌍 Environment Variables

### Core
| Key | Description | Required? |
| :--- | :--- | :--- |
| `APP_SECRET_KEY` | Secret key for admin actions (sending mail) | Yes |
| `WIDGET_API_KEY` | Public key for the widget (view only) | Yes |
| `BUBBLE_API_TOKEN` | Bearer token for Bubble Workflow API requests | Yes |
| `BUBBLE_APP_URL` | Base URL of your app (e.g., `https://my-app.bubbleapps.io`) | Yes |
| `ALLOWED_ORIGINS` | Comma-separated list of domains for CORS | No (defaults to `*`) |

### Providers
| Key | Description |
| :--- | :--- |
| `OUTLOOK_CLIENT_ID` | Client ID from Azure Portal for Outlook |
| `OUTLOOK_CLIENT_SECRET` | Client Secret from Azure Portal |
| `POSTMARK_API_TOKEN` | Server token for sending via Postmark |

---

## 🔄 Bubble.io Integration

The proxy actively uses `wf` (Backend Workflows) endpoints in Bubble:

*   **`get_quote_preview`**: Called before sending. Bubble should return the email's HTML code and (optionally) the body text.
*   **`send_quote`**: Method for preparing data before sending. The proxy passes the quote ID, receiving back the final HTML and the URL of the generated PDF.
*   **`send_remember`**: Notifies Bubble that the email was successfully sent via the proxy so Bubble can schedule reminders.

### Feedback Webhook (Reminder Webhook)
*   **Endpoint:** `POST /api/webhook/reminder`
*   **Purpose:** Bubble calls this endpoint when an automatic reminder triggers. The proxy takes the HTML from the request and physically sends the email via Gmail/Outlook.

---

## 📡 Main Proxy API Endpoints

### Email
- `GET /api/messages`: List emails (supports `provider=gmail|outlook`).
- `GET /api/messages/:id`: Get full email content with parsed MIME (text, HTML, attachment list).
- `POST /api/messages/send`: Send a message (requires Admin API Key).

### Quote-Specific
- `POST /api/quote/preview`: Get HTML preview from Bubble.
- `POST /api/quote/send`: Complex process: get HTML from Bubble -> download PDF -> send via chosen provider -> notify Bubble of success.

---

## 🚀 Deployment (Easypanel / Docker)

### Dockerfile
Uses a multi-stage build:
1.  **Frontend Builder:** Builds the React/Vite frontend.
2.  **Backend Builder:** Compiles the Rust binary.
3.  **Runtime:** Minimal Debian image with CA certificates installed.

---
*Documentation updated February 2026.*
