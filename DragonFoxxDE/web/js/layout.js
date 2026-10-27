// Layout: Drag tabs to reorder, drag the explorer edge to resize,
// Ctrl+B / Ctrl+J to hide or show the explorer and bottom panel

const SIDEBAR_WIDTH_KEY = "dragonide_sidebar_width";
const SIDEBAR_HIDDEN_KEY = "dragonide_sidebar_hidden";
const PANEL_HIDDEN_KEY = "dragonide_panel_hidden";

let tabDrag = null;

// drag tabs

function tabDropIndex(x) {
    // the dragged tab goes before the first tab whose middle is right of the mouse
    const elements = [...document.querySelectorAll("#tabs .tab[data-path]")];

    for (let i = 0; i < elements.length; i++) {
        const rect = elements[i].getBoundingClientRect();
        if (x < rect.left + rect.width / 2) return i;
    }

    return elements.length;
}

function showTabDropMarker(x) {
    const elements = [...document.querySelectorAll("#tabs .tab[data-path]")];
    const index = tabDropIndex(x);

    elements.forEach((element, i) => {
        element.classList.toggle("dragging", element.dataset.path === tabDrag.path);
        element.classList.toggle("drop-before", i === index);
        element.classList.toggle("drop-after", i === elements.length - 1 && index === elements.length);
    });
}

function moveTab(path, toIndex) {
    const tabsList = window.openTabs || (typeof openTabs !== "undefined" ? openTabs : null);
    if (!tabsList) return;

    const from = tabsList.findIndex(tab => tab.path === path);
    if (from === -1) return;

    if (toIndex === from || toIndex === from + 1) return;

    const [tab] = tabsList.splice(from, 1);
    if (toIndex > from) toIndex--; // the list is one shorter

    tabsList.splice(toIndex, 0, tab);

    if (typeof renderTabs === "function") {
        renderTabs();
    } else if (typeof window.renderTabs === "function") {
        window.renderTabs();
    }
}

function stopTabDrag() {
    tabDrag = null;
    document.body.style.cursor = "";
    document.body.style.userSelect = "";
    document.querySelectorAll("#tabs .tab").forEach(element => {
        element.classList.remove("dragging", "drop-before", "drop-after");
    });
}

function initTabDrag() {
    const tabs = document.getElementById("tabs");
    if (!tabs) return;

    // the mouse wheel scrolls a long row of tabs sideways
    tabs.addEventListener("wheel", (e) => {
        tabs.scrollLeft += e.deltaY;
    }, { passive: true });

    tabs.addEventListener("mousedown", (e) => {
        const tab = e.target.closest(".tab[data-path]");
        if (!tab || e.button !== 0 || e.target.closest(".tab-close")) return;

        tabDrag = { path: tab.dataset.path, startX: e.clientX, moved: false };
    });

    document.addEventListener("mousemove", (e) => {
        if (!tabDrag) return;

        // a tiny move is still a normal click
        if (!tabDrag.moved && Math.abs(e.clientX - tabDrag.startX) < 5) return;

        tabDrag.moved = true;
        e.preventDefault();
        document.body.style.cursor = "grabbing";
        document.body.style.userSelect = "none";
        showTabDropMarker(e.clientX);
    });

    document.addEventListener("mouseup", (e) => {
        if (!tabDrag) return;

        if (tabDrag.moved) {
            moveTab(tabDrag.path, tabDropIndex(e.clientX));
        }
        stopTabDrag();
    });

    window.addEventListener("blur", stopTabDrag);
}

// resize explorer

function initSidebarResize() {
    const sidebar = document.getElementById("sidebar");
    const resizer = document.getElementById("sidebar-resizer");

    if (!sidebar || !resizer) return;

    const saved = parseInt(localStorage.getItem(SIDEBAR_WIDTH_KEY), 10);
    if (saved) {
        sidebar.style.width = Math.max(160, Math.min(saved, window.innerWidth - 400)) + "px";
    }

    let dragging = false;

    function stopDragging() {
        if (!dragging) return;

        dragging = false;
        resizer.classList.remove("dragging");
        document.body.style.cursor = "";
        document.body.style.userSelect = "";
        localStorage.setItem(SIDEBAR_WIDTH_KEY, parseInt(sidebar.style.width, 10));
    }

    resizer.addEventListener("mousedown", (e) => {
        e.preventDefault();
        dragging = true;
        resizer.classList.add("dragging");
        document.body.style.cursor = "ew-resize";
        document.body.style.userSelect = "none";
    });

    document.addEventListener("mousemove", (e) => {
        if (!dragging) return;

        const left = sidebar.getBoundingClientRect().left;
        const width = Math.max(160, Math.min(e.clientX - left, window.innerWidth - 400));
        sidebar.style.width = width + "px";
    });

    document.addEventListener("mouseup", stopDragging);
    window.addEventListener("blur", stopDragging);

    resizer.addEventListener("dblclick", () => {
        sidebar.style.width = "";
        localStorage.removeItem(SIDEBAR_WIDTH_KEY);
    });
}

// hide / show
function setSidebarVisible(visible) {
    const sidebar = document.getElementById("sidebar");
    const resizer = document.getElementById("sidebar-resizer");
    if (!sidebar) return;

    sidebar.style.display = visible ? "" : "none";
    if (resizer) resizer.style.display = visible ? "" : "none";
    localStorage.setItem(SIDEBAR_HIDDEN_KEY, visible ? "0" : "1");
}

function setBottomPanelVisible(visible) {
    const panel = document.getElementById("bottom-panel");
    if (!panel) return;

    panel.style.display = visible ? "" : "none";
    localStorage.setItem(PANEL_HIDDEN_KEY, visible ? "0" : "1");
}

function initLayoutToggles() {
    if (localStorage.getItem(SIDEBAR_HIDDEN_KEY) === "1") setSidebarVisible(false);
    if (localStorage.getItem(PANEL_HIDDEN_KEY) === "1") setBottomPanelVisible(false);

    document.addEventListener("keydown", (e) => {
        if (!(e.ctrlKey || e.metaKey) || e.shiftKey || e.altKey) return;

        const key = e.key.toLowerCase();

        if (key === "b") {
            e.preventDefault();
            const sidebar = document.getElementById("sidebar");
            setSidebarVisible(sidebar?.style.display === "none");
        } else if (key === "j") {
            e.preventDefault();
            const panel = document.getElementById("bottom-panel");
            setBottomPanelVisible(panel?.style.display === "none");
        }
    });

    // clicking explorer / search brings the sidebar back
    document.querySelectorAll("#activity-bar .activity-button:not(#settings-button)").forEach(button => {
        button.addEventListener("click", () => setSidebarVisible(true));
    });
}

function initLayout() {
    initTabDrag();
    initSidebarResize();
    initLayoutToggles();
}

if (document.readyState === "loading") {
    document.addEventListener("DOMContentLoaded", initLayout);
} else {
    initLayout();
}

window.setSidebarVisible = setSidebarVisible;
window.setBottomPanelVisible = setBottomPanelVisible;