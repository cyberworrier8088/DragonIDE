let quickOpenTimer = null;
let quickOpenRequestId = 0;
let quickOpenResults = [];
let quickOpenSelected = 0;
let quickOpenSearch = null; // the newest search, so Enter can wait for its answer

function openQuickOpen() {
    const modal = document.getElementById("quick-open-modal");
    const input = document.getElementById("quick-open-input");
    if (!modal || !input) return;

    modal.classList.remove("hidden");
    input.value = "";
    input.focus();
    quickOpenSearch = runQuickOpen();
}

function closeQuickOpen() {

    document.getElementById("quick-open-modal")?.classList.add("hidden");
}

async function runQuickOpen() {
    const input = document.getElementById("quick-open-input");
    const list = document.getElementById("quick-open-results");
    if (!input || !list) return;

    const requestId = ++quickOpenRequestId;
    const workspace = window.currentWorkspace;

    if (!workspace) {
        quickOpenResults = [];
        list.innerHTML = `<div class="quick-open-empty">Open a folder first.</div>`;
        return;
    }

    try {
        const { invoke } = window.__TAURI__.core;
        const results = await invoke("find_files", {
            root: workspace,
            query: input.value
        });

        if (requestId !== quickOpenRequestId) return;
        quickOpenResults = results;
        quickOpenSelected = 0;
        renderQuickOpen();
    } catch (error) {

        if (requestId !== quickOpenRequestId) return;
        list.textContent = `Failed: ${error}`;
    }
}

function renderQuickOpen() {

    const list = document.getElementById("quick-open-results");
    list.innerHTML = "";

    if (quickOpenResults.length === 0) {
        list.innerHTML = `<div class="quick-open-empty">No results.</div>`;
        return;
    }

    quickOpenResults.forEach((file, index) => {

        const row = document.createElement("div");
        row.className = "quick-open-item";
        if (index === quickOpenSelected) row.classList.add("selected");

        const name = document.createElement("span");
        name.className = "quick-open-name";
        name.textContent = file.name;

        const path = document.createElement("span");
        path.className = "quick-open-path";
        path.textContent = file.relative;

        row.appendChild(name);
        row.appendChild(path);
        row.addEventListener("click", () => pickQuickOpen(index));
        list.appendChild(row);
    });

    list.children[quickOpenSelected]?.scrollIntoView({ block: "nearest" });
}

function moveQuickOpenSelection(step) {
    if (quickOpenResults.length === 0) return;

    quickOpenSelected = (quickOpenSelected + step + quickOpenResults.length) % quickOpenResults.length;
    renderQuickOpen();
}

async function pickQuickOpen(index) {
    const file = quickOpenResults[index];
    if (!file) return;

    closeQuickOpen();
    await window.openFile({ name: file.name, path: file.path, is_directory: false });
}

function initQuickOpen() {
    const input = document.getElementById("quick-open-input");

    input?.addEventListener("input", () => {
        clearTimeout(quickOpenTimer);
        quickOpenTimer = setTimeout(() => {

            quickOpenTimer = null;
            quickOpenSearch = runQuickOpen();
        }, 150);
    });

    input?.addEventListener("keydown", async (e) => {

        if (e.key === "ArrowDown") {
            e.preventDefault();
            moveQuickOpenSelection(1);
        } else if (e.key === "ArrowUp") {

            e.preventDefault();
            moveQuickOpenSelection(-1);
        } else if (e.key === "Enter") {

            e.preventDefault();
            // typed fast and pressed enter before the search, or before its answer came back
            if (quickOpenTimer) {
                clearTimeout(quickOpenTimer);
                quickOpenTimer = null;
                quickOpenSearch = runQuickOpen();
            }
            await quickOpenSearch;
            pickQuickOpen(quickOpenSelected);
        } else if (e.key === "Escape") {
            e.preventDefault();
            closeQuickOpen();
        }
    });

    document.querySelector("#quick-open-modal .modal-backdrop")?.addEventListener("click", closeQuickOpen);
    document.addEventListener("keydown", (e) => {
        if ((e.ctrlKey || e.metaKey) && !e.shiftKey && e.key.toLowerCase() === "p") {
            e.preventDefault(); // stops the print dialog
            openQuickOpen();
        }
    });
}

if (document.readyState === "loading") {
    document.addEventListener("DOMContentLoaded", initQuickOpen);
} else {
    initQuickOpen();
}

window.openQuickOpen = openQuickOpen;