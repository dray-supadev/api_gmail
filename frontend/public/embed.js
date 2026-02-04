(function () {
    const SCRIPT_ID = "gmail-outlook-widget-script";
    if (document.getElementById(SCRIPT_ID)) return;

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

                // Add close button
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

                document.body.appendChild(container);
            }

            // Create Iframe
            const iframe = document.createElement("iframe");
            // TODO: Replace with actual deployed URL
            iframe.src = "http://localhost:5173";
            iframe.style.width = "100%";
            iframe.style.height = "100%";
            iframe.style.border = "none";

            container.appendChild(iframe);
        }
    };
})();
