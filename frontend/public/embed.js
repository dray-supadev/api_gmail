(function () {
    if (window.GmailOutlookWidget) {
        console.log("Widgets already loaded");
        return;
    }

    window.GmailOutlookWidget = {
        open: function (config) {
            // Configuration
            const width = config.width || "100%";
            const height = config.height || "600px";
            const containerId = config.containerId || "gmail-outlook-widget-container";

            let container = document.getElementById(containerId);
            if (!container) {
                // If no container specified, create a modal overlay
                container = document.createElement("div");
                container.id = containerId;
                container.style.position = "fixed";
                container.style.top = "50%";
                container.style.left = "50%";
                container.style.transform = "translate(-50%, -50%)";
                container.style.width = width;
                container.style.height = height;
                container.style.zIndex = "9999";
                container.style.boxShadow = "0 10px 25px rgba(0,0,0,0.5)";
                container.style.borderRadius = "12px";
                container.style.overflow = "hidden";
                container.style.backgroundColor = "white";

                // Close button removed in favor of internal app close button
                /*
                const closeBtn = document.createElement("button");
                closeBtn.innerHTML = "&times;";
                closeBtn.style.position = "absolute";
                closeBtn.style.top = "10px";
                closeBtn.style.right = "15px";
                closeBtn.style.background = "transparent";
                closeBtn.style.border = "none";
                closeBtn.style.fontSize = "24px";
                closeBtn.style.cursor = "pointer";
                closeBtn.onclick = function () {
                    document.body.removeChild(container);
                };
                container.appendChild(closeBtn);
                */

                document.body.appendChild(container);
            }

            // Create Iframe
            // Create Iframe
            const iframe = document.createElement("iframe");

            // Auto-detect domain from the script tag source, or fallback to config
            let appUrl = config.appUrl;
            if (!appUrl) {
                // Try to find the script tag that loaded this file
                const scriptTag = document.querySelector('script[src*="embed.js"]');
                if (scriptTag) {
                    try {
                        const url = new URL(scriptTag.src);
                        appUrl = url.origin;
                    } catch (e) {
                        console.error("Could not parse script URL", e);
                    }
                }
            }

            iframe.src = appUrl || "https://app.drayinsight.com";

            // Append configuration params
            const url = new URL(iframe.src);

            // Pass specific tokens if available
            if (config.gmailToken) url.searchParams.set("gmailToken", config.gmailToken);
            if (config.outlookToken) url.searchParams.set("outlookToken", config.outlookToken);

            // Legacy single token support (optional, or if user only sends one)
            if (config.token) url.searchParams.set("token", config.token);

            if (config.provider) url.searchParams.set("provider", config.provider);
            if (config.quoteId) url.searchParams.set("quoteId", config.quoteId);
            if (config.bubbleVersion) url.searchParams.set("bubbleVersion", config.bubbleVersion);
            if (config.pdfExportSettings) {
                // Assuming it's an array of strings
                if (Array.isArray(config.pdfExportSettings)) {
                    url.searchParams.set("pdfExportSettings", config.pdfExportSettings.join(","));
                } else {
                    url.searchParams.set("pdfExportSettings", config.pdfExportSettings);
                }
            }
            iframe.src = url.toString();

            iframe.style.width = "100%";
            iframe.style.height = "100%";
            iframe.style.border = "none";

            container.appendChild(iframe);
        }
    };
    // Listen for close message from the iframe
    window.addEventListener("message", function (event) {
        if (event.data && event.data.type === "GMAIL_WIDGET_CLOSE") {
            const container = document.getElementById("gmail-outlook-widget-container");
            if (container) {
                document.body.removeChild(container);
            }
        }
    });
})();
