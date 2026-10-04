const PANEL_HEIGHT_KEY = "dragonide_panel_height";

function initPanelResize() {
    const panel = document.getElementById("bottom-panel");
    const resizer = document.getElementById("panel-resizer");
    const main = document.getElementById("main");
    const statusBar = document.getElementById("status-bar");

    if (!panel || !resizer || !main) return;

    const saved = parseInt(localStorage.getItem(PANEL_HEIGHT_KEY), 10);

    if (saved) {
        panel.style.height = Math.max(100, Math.min(saved, window.innerHeight - 250)) + "px";
    }

    let dragging = false;

    resizer.addEventListener("mousedown", (e) => {
        e.preventDefault();
        dragging = true;
        resizer.classList.add("dragging");
        document.body.style.cursor = "ns-resize";
        document.body.style.userSelect = "none";
    });

    document.addEventListener("mousemove", (e) => {
        if (!dragging) return;

        const mainRect = main.getBoundingClientRect();
        const statusHeight = statusBar ? statusBar.offsetHeight : 0;
        const maxHeight = mainRect.height - 150;

        let newHeight = mainRect.bottom - statusHeight - e.clientY;
        newHeight = Math.max(100, Math.min(newHeight, maxHeight));
        panel.style.height = newHeight + "px";
    });
}

if (document.readyState === "loading") {
    document.addEventListener("DOMContentLoaded", initPanelResize);
} else {
    initPanelResize();
}