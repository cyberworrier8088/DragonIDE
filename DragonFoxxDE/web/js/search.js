let searchTimer = null;
let searchRequestId = 0;

function getWorkspace() {
    if (typeof window !== "undefined" && window.currentWorkspace) {
        return window.currentWorkspace;
    }
    try {
        if (typeof currentWorkspace !== "undefined" && currentWorkspace) {
            return currentWorkspace;
        }
    } catch (_) {}
    return null;
}

function showSidebarView(view) {
    const explorer = document.getElementById("file-explorer");
    const searchView = document.getElementById("search-view");
    const title = document.getElementById("sidebar-title") || document.querySelector(".sidebar-header span");
    const isSearch = view === "search";

    if (explorer) explorer.style.display = isSearch ? "none" : "block";
    if (searchView) searchView.style.display = isSearch ? "flex" : "none";

    if (title) {
        const key = isSearch ? "search" : "explorer";
        title.setAttribute("data-i18n", key);
        title.textContent = typeof t === "function" ? t(key) : key.toUpperCase();
    }

    const explorerBtn = document.getElementById("explorer-activity-btn") || document.querySelector("#activity-bar .activity-button:first-child");
    const searchBtn = document.getElementById("search-activity-btn") || document.querySelectorAll("#activity-bar .activity-button")[1];

    if (explorerBtn) explorerBtn.classList.toggle("active", !isSearch);
    if (searchBtn) searchBtn.classList.toggle("active", isSearch);

    if (isSearch) {
        const input = document.getElementById("search-input");
        input?.focus();
        input?.select();
    }
}

async function runSearch() {
    const input = document.getElementById("search-input");
    const caseCheck = document.getElementById("search-case-check");
    const summary = document.getElementById("search-summary");
    const resultsBox = document.getElementById("search-results");

    if (!input || !summary || !resultsBox) return;

    const query = input.value;
    const caseSensitive = caseCheck ? caseCheck.checked : false;
    const requestId = ++searchRequestId;

    resultsBox.innerHTML = "";

    if (!query.trim()) {
        summary.textContent = "";
        return;
    }

    const workspace = getWorkspace();
    if (!workspace) {
        summary.textContent = "Open a folder first.";
        return;
    }

    summary.textContent = "Searching...";

    try {
        const { invoke } = window.__TAURI__.core;
        const results = await invoke("search_workspace", {
            root: workspace,
            query: query,
            caseSensitive: caseSensitive
        });

        if (requestId !== searchRequestId) return;
        renderSearchResults(results, query, caseSensitive);
    } catch (error) {
        if (requestId !== searchRequestId) return;
        summary.textContent = `Search failed: ${error}`;
    }
}

function renderSearchResults(results, query, caseSensitive) {
    const summary = document.getElementById("search-summary");
    const resultsBox = document.getElementById("search-results");
    resultsBox.innerHTML = "";

    if (results.length === 0) {
        summary.textContent = "No results found";
        return;
    }

    const groups = new Map();
    for (const result of results) {
        if (!groups.has(result.path)) groups.set(result.path, []);
        groups.get(result.path).push(result);
    }

    const limitNote = results.length >= 1000 ? " (showing first 1000)" : "";
    summary.textContent = `${results.length} results in ${groups.size} files${limitNote}`;

    for (const [path, matches] of groups) {
        const group = document.createElement("div");
        group.className = "search-file-group";

        const header = document.createElement("div");
        header.className = "search-file-name";
        header.title = path;
        header.textContent = matches[0].name;

        const count = document.createElement("span");
        count.className = "search-file-count";
        count.textContent = matches.length;
        header.appendChild(count);
        group.appendChild(header);

        for (const match of matches) {
            const row = document.createElement("div");
            row.className = "search-result";

            const lineNo = document.createElement("span");
            lineNo.className = "search-line-number";
            lineNo.textContent = match.line;

            const text = document.createElement("span");
            text.className = "search-line-text";
            appendHighlightedText(text, match.text, query, caseSensitive);

            row.appendChild(lineNo);
            row.appendChild(text);
            row.addEventListener("click", () => openSearchResult(match));
            group.appendChild(row);
        }

        resultsBox.appendChild(group);
    }
}

function appendHighlightedText(container, text, query, caseSensitive) {
    const haystack = caseSensitive ? text : text.toLowerCase();
    const needle = caseSensitive ? query : query.toLowerCase();
    let index = 0;

    while (needle.length > 0) {
        const found = haystack.indexOf(needle, index);
        if (found === -1) break;

        container.appendChild(document.createTextNode(text.slice(index, found)));

        const mark = document.createElement("span");
        mark.className = "search-match";
        mark.textContent = text.slice(found, found + needle.length);
        container.appendChild(mark);

        index = found + needle.length;
    }

    container.appendChild(document.createTextNode(text.slice(index)));
}

async function openSearchResult(match) {
    const fn = (typeof openFile === "function") ? openFile : window.openFile;
    if (fn) {
        await fn({ name: match.name, path: match.path, is_directory: false });
    }
    const jump = (typeof goToLineNumber === "function") ? goToLineNumber : window.goToLineNumber;
    if (jump) {
        jump(match.line);
    }
    const upd = (typeof updateCursorPosition === "function") ? updateCursorPosition : window.updateCursorPosition;
    if (upd) {
        upd();
    }
    setTimeout(() => {
        if (jump) jump(match.line);
    }, 50);
}

function initSearch() {
    const explorerBtn = document.getElementById("explorer-activity-btn") || document.querySelector("#activity-bar .activity-button:first-child");
    const searchBtn = document.getElementById("search-activity-btn") || document.querySelectorAll("#activity-bar .activity-button")[1];

    explorerBtn?.addEventListener("click", () => showSidebarView("explorer"));
    searchBtn?.addEventListener("click", () => showSidebarView("search"));

    const input = document.getElementById("search-input");
    input?.addEventListener("input", () => {
        clearTimeout(searchTimer);
        searchTimer = setTimeout(runSearch, 250);
    });
    input?.addEventListener("keydown", (e) => {
        if (e.key === "Enter") {
            clearTimeout(searchTimer);
            runSearch();
        }
    });

    document.getElementById("search-case-check")?.addEventListener("change", runSearch);

    document.addEventListener("keydown", (e) => {
        if ((e.ctrlKey || e.metaKey) && e.shiftKey && e.key.toLowerCase() === "f") {
            e.preventDefault();
            showSidebarView("search");
        }
    });
}

if (document.readyState === "loading") {
    document.addEventListener("DOMContentLoaded", initSearch);
} else {
    initSearch();
}

window.showSidebarView = showSidebarView;
window.runSearch = runSearch;