const { invoke } = window.__TAURI__.core;

let currentFile = null;
let currentFileContent = "";
let editorChangeTimer = null;
let openTabs = [];
let currentWorkspace = null;

let undoStack = [];
let redoStack = [];
let isUndoRedo = false;
let lastSavedState = null;

function saveCurrentTabHistory() {
    if (currentFile) {
        const tab = openTabs.find(t => t.path === currentFile.path);
        if (tab) {
            tab.undoStack = [...undoStack];
            tab.redoStack = [...redoStack];
            tab.lastSavedState = lastSavedState ? { ...lastSavedState } : null;
        }
    }
}

function loadTabHistory(tab) {
    if (tab) {
        undoStack = tab.undoStack || [];
        redoStack = tab.redoStack || [];
        lastSavedState = tab.lastSavedState || null;
    } else {
        undoStack = [];
        redoStack = [];
        lastSavedState = null;
    }
}


async function startDragonIDE() {
    if (typeof applyAllSettings === "function") {
        applyAllSettings();
    }

    try {
        const name = await invoke("get_ide_name");
        console.log("Rust says: ", name);

        const count = await invoke("document_count");
        console.log("Open Document: ", count);
    } catch (error) {
        console.error("Error during startup:", error);
    }

    // --- CONTEXT MENU ACTION LISTENERS ---

    // New File
    document.getElementById("ctx-new-file")?.addEventListener("click", async () => {
        if (!selectedContextEntry) return;
        hideContextMenu();

        const targetDir = selectedContextEntry.is_directory
            ? selectedContextEntry.path
            : getParentPath(selectedContextEntry.path);

        const fileName = prompt("Enter new file name:");
        if (!fileName) return;

        const newPath = joinPath(targetDir, fileName);
        try {
            await invoke("create_file", { path: newPath });
            await refreshWorkspace();
            hideContextMenu();
        } catch (err) {
            console.error("Error creating file:", err);
        }
    });

    // New Folder
    document.getElementById("ctx-new-folder")?.addEventListener("click", async () => {
        if (!selectedContextEntry) return;
        hideContextMenu();

        const targetDir = selectedContextEntry.is_directory
            ? selectedContextEntry.path
            : getParentPath(selectedContextEntry.path);

        const folderName = prompt("Enter new folder name:");
        if (!folderName) return;

        const newPath = joinPath(targetDir, folderName);
        try {
            await invoke("create_directory", { path: newPath });
            await refreshWorkspace();
            hideContextMenu();
        } catch (err) {
            console.error("Error creating folder:", err);
        }
    });

    // Rename
    document.getElementById("ctx-rename")?.addEventListener("click", async () => {
        if (!selectedContextEntry) return;
        hideContextMenu();

        const oldPath = selectedContextEntry.path;
        const currentName = selectedContextEntry.name;
        const newName = prompt("Enter new name:", currentName);

        if (!newName || newName === currentName) return;

        const parentDir = getParentPath(oldPath);
        const newPath = joinPath(parentDir, newName);

        try {
            await invoke("rename_entry", { oldPath, newPath });
            await refreshWorkspace();
            hideContextMenu();
        } catch (err) {
            console.error("Error renaming entry:", err);
        }
    });

    // Delete
    document.getElementById("ctx-delete")?.addEventListener("click", async () => {
        if (!selectedContextEntry) return;
        hideContextMenu();

        const confirmDelete = confirm(`Are you sure you want to delete "${selectedContextEntry.name}"?`);
        if (!confirmDelete) return;

        try {
            await invoke("delete_entry", {
                path: selectedContextEntry.path,
                isDirectory: selectedContextEntry.is_directory
            });

            await closeTab(selectedContextEntry.path, true);
            await refreshWorkspace();
            hideContextMenu();
        } catch (err) {
            console.error("Error deleting entry:", err);
        }
    });

    // Copy Path
    document.getElementById("ctx-copy-path")?.addEventListener("click", async () => {
        if (!selectedContextEntry) return;
        hideContextMenu();
        await navigator.clipboard.writeText(selectedContextEntry.path);
        hideContextMenu();
    });
}

async function openFolder() {
    try {

        const folder = await window.__TAURI__.dialog.open({
            directory: true,
            multiple: false
        });

        if (folder === null) {
            console.log("Folder selection cancelled");
            return;
        }

        currentWorkspace = folder;

        console.log("Selected workspace: ", folder);

        const explorerName =
            document.getElementById("workspace-name");

        if (explorerName) {
            explorerName.textContent =
                folder.split(/[\\/]/).pop();
        }

        await refreshWorkspace();

    } catch (error) {
        console.error("Failed to open folder: ", error);
    }
}

async function refreshWorkspace() {
    if (!currentWorkspace) return;

    const files = await invoke("read_workspace", { path: currentWorkspace });
    renderFileTree(files);
}


function createFileEntry(entry) {

    const element = document.createElement("div");

    element.classList.add("file-entry");

    if (entry.is_directory) {
        element.classList.add("directory");
    }

    // RIGHT-CLICK CONTEXT MENU EVENT
    element.addEventListener("contextmenu", (event) => {
        showContextMenu(event, entry);
    });

    const arrow = document.createElement("span");
    arrow.className = "file-arrow";

    const icon = document.createElement("span");
    icon.className = "file-icon";

    const name = document.createElement("span");
    name.className = "file-name";


    name.textContent = entry.name;

    if (entry.is_directory) {
        arrow.classList.add("collapsed");
        icon.classList.add("folder");

        element.addEventListener("click", () => {
            toggleDirectory(element, entry, arrow);
        });
    } else {
        icon.classList.add("file");

        element.addEventListener("click", () => {
            openFile(entry);
        });
    }


    element.appendChild(arrow);
    element.appendChild(icon);
    element.appendChild(name);

    return element;


}

async function toggleDirectory(element, entry, arrow) {

    const existingChildren = element.nextElementSibling;

    if (
        existingChildren && existingChildren.classList.contains("file-children")
    ) {

        existingChildren.remove();

        arrow.classList.add("collapsed");
        arrow.classList.remove("expanded");

        return;
    }

    try {

        arrow.classList.remove("collapsed");
        arrow.classList.add("expanded");

        const files = await invoke("read_workspace", {
            path: entry.path
        });

        const children = document.createElement("div");

        children.className = "file-children";

        for (const file of files) {

            const child = createFileEntry(file);

            children.appendChild(child);
        }

        element.after(children);
    } catch (error) {
        console.error(
            "Failed to read directory: ",
            error
        );

        arrow.classList.add("collapsed");
        arrow.classList.remove("expanded");
    }
}


async function openFile(entry) {

    try {

        console.log("Opening fille:", entry.path);

        const content = await invoke("open_document", {
            path: entry.path
        });

        saveCurrentTabHistory();

        const existingTab = openTabs.find(t => t.path === entry.path);
        if (existingTab) {
            loadTabHistory(existingTab);
        } else {
            undoStack = [];
            redoStack = [];
            lastSavedState = {
                text: content,
                selectionStart: 0,
                selectionEnd: 0
            };
        }

        currentFile = entry;
        currentFileContent = content;

        const welcome = document.getElementById("welcome-screen");

        const editorContainer = document.getElementById("editor-container");
        const editor = document.getElementById("code-editor");

        if (welcome) {
            welcome.style.display = "none";
        }

        if (editorContainer) {
            editorContainer.style.display = "flex";
        }

        if (editor) {
            editor.value = content;
            syncHighlight();
            updateLineNumbers();
            editor.focus();
        }


        addOrActivateTab(entry);

        const fileName = entry.path.split(/[\\/]/).pop();
        window.dispatchEvent(new CustomEvent("file-opened", { detail: fileName }));

        console.log("Opened: ", entry.name);

        const count = await invoke("document_count");

        console.log("Open documents:", count);

    } catch (error) {

        console.error(
            "Failed to open file:",
            error
        );
    }
}


function updateEditorTab(fileName, modified = false) {

    const tabs = document.getElementById("tabs");

    const marker = modified ? " *" : "";

    tabs.innerHTML = `
        <div class="tab active">
            <span>${escapeHtml(fileName + marker)}</span>
            <span class="tab-close">×</span>
        </div>
    `;

}

function escapeHtml(value) {

    const element = document.createElement("div");

    element.textContent = value;

    return element.innerHTML;
}

function renderFileTree(files) {

    const tree = document.getElementById("file-tree");

    tree.innerHTML = "";

    for (const file of files) {
        const entry = createFileEntry(file);

        tree.appendChild(entry);
    }
}

async function updateCurrentDocument() {

    if (!currentFile) {
        return;
    }

    const editor = document.getElementById("code-editor");

    if (!editor) {
        return;
    }


    try {
        await invoke("update_document", {
            path: currentFile.path,
            text: editor.value
        });

        currentFileContent = editor.value;

        const tab = openTabs.find(
            tab => tab.path === currentFile.path
        );

        if (tab) {
            tab.modified = true;
        }

        renderTabs();
    } catch (error) {

        console.error(
            "Failed to update document:",
            error
        );
    }
}


function addOrActivateTab(entry) {

    const existing = openTabs.find(
        tab => tab.path === entry.path
    );

    if (!existing) {
        openTabs.push({
            path: entry.path,
            name: entry.name,
            modified: false,
            undoStack: [...undoStack],
            redoStack: [...redoStack],
            lastSavedState: lastSavedState ? { ...lastSavedState } : null
        });
    }

    renderTabs();
}


function renderTabs() {
    const tabs = document.getElementById("tabs");

    tabs.innerHTML = "";

    for (const tab of openTabs) {
        const element = document.createElement("div");

        element.classList = "tab";


        if (
            currentFile && currentFile.path === tab.path
        ) {
            element.classList.add("active");
        }

        const name = document.createElement("span");

        name.textContent = tab.name + (tab.modified ? " *" : "");

        const close = document.createElement("span");

        close.className = "tab-close";
        close.textContent = "×";

        element.appendChild(name);
        element.appendChild(close);

        element.addEventListener("click", () => {
            activateTab(tab.path);
        });

        close.addEventListener("click", async (event) => {
            event.stopPropagation();

            await closeTab(tab.path);
        });

        tabs.appendChild(element);
    }
}


async function activateTab(path) {

    const tab = openTabs.find(
        tab => tab.path === path
    );

    if (!tab) {
        return;
    }

    saveCurrentTabHistory();

    try {

        const content = await invoke("open_document", {
            path: tab.path
        });

        currentFile = {
            name: tab.name,
            path: tab.path
        };

        currentFileContent = content;

        loadTabHistory(tab);
        if (!lastSavedState) {
            lastSavedState = {
                text: content,
                selectionStart: 0,
                selectionEnd: 0
            };
        }

        window.dispatchEvent(new CustomEvent("file-opened", { detail: tab.name }));

        const editorContainer = document.getElementById("editor-container");
        const editor = document.getElementById("code-editor");
        const welcome = document.getElementById("welcome-screen");

        if (welcome) {
            welcome.style.display = "none";
        }

        if (editorContainer) {
            editorContainer.style.display = "flex";
        }

        if (editor) {
            editor.value = content;

            updateLineNumbers();
            updateCursorPosition();
            syncHighlight();

            editor.focus();
        }

        renderTabs();
    } catch (error) {
        console.error(
            "Failed to activate tab: ",
            error
        );
    }

}

async function closeTab(path, force = false) {
    const index = openTabs.findIndex(
        tab => tab.path === path
    );


    if (index === -1) {
        return;
    }

    const tab = openTabs[index];

    if (tab.modified && !force) {
        let confirmClose = false;

        if (window.__TAURI__ && window.__TAURI__.dialog) {

            confirmClose = await window.__TAURI__.dialog.confirm(
                t("unsavedChangesMsg", { fileName: tab.name }),
                { title: "DragonFoxIDE", kind: "warning" }
            );
        } else {

            confirmClose = confirm(`"${tab.name}" has unsaved changes. Do you want to close it without saving?`);
        }

        if (!confirmClose) {
            return;
        }
    }

    const wasActive = currentFile && currentFile.path === path;


    openTabs.splice(index, 1);

    if (wasActive) {
        if (openTabs.length === 0) {
            currentFile = null;
            currentFileContent = "";
            undoStack = [];
            redoStack = [];
            lastSavedState = null;


            const editorContainer = document.getElementById("editor-container");
            const editor = document.getElementById("code-editor");
            const welcome = document.getElementById("welcome-screen");

            if (editorContainer) {
                editorContainer.style.display = "none";

            }

            if (editor) {
                editor.value = "";
            }

            if (welcome) {
                welcome.style.display = "block";
            }

            window.dispatchEvent(new CustomEvent("file-opened", { detail: null }));

        } else {
            const nextIndex = Math.min(index, openTabs.length - 1);

            activateTab(openTabs[nextIndex].path)
        }
    }

    renderTabs();
}


function updateLineNumbers() {

    const editor = document.getElementById("code-editor");

    const lineNumbers = document.getElementById("line-numbers");

    if (!editor || !lineNumbers) {
        return;
    }

    const lineCount = editor.value.split("\n").length;

    let html = "";

    for (let i = 1; i <= lineCount; i++) {
        html += `<div>${i}</div>`;
    }

    lineNumbers.innerHTML = html;
}


function updateCursorPosition() {
    const editor = document.getElementById("code-editor");

    const position = document.getElementById("cursor-position");

    if (!editor || !position) {
        return;
    }

    const cursor = editor.selectionStart;

    const beforeCursor = editor.value.slice(0, cursor);

    const lines = beforeCursor.split("\n");

    const line = lines.length;

    const column = lines[lines.length - 1].length + 1;

    position.textContent = `Ln ${line}, Col ${column}`;

    if (lastSavedState && lastSavedState.text === editor.value) {
        lastSavedState.selectionStart = editor.selectionStart;
        lastSavedState.selectionEnd = editor.selectionEnd;
    }
}



function syncEditorScroll() {
    const editor = document.getElementById("code-editor");

    const lineNumbers = document.getElementById("line-numbers");

    if (!editor || !lineNumbers) {
        return;
    }

    lineNumbers.scrollTop = editor.scrollTop;
}



function undoEdit() {
    const editor = document.getElementById("code-editor");
    if (!editor || undoStack.length === 0) {
        return;
    }

    isUndoRedo = true;

    redoStack.push({
        text: editor.value,
        selectionStart: editor.selectionStart,
        selectionEnd: editor.selectionEnd
    });

    const previousState = undoStack.pop();
    editor.value = previousState.text;
    editor.selectionStart = previousState.selectionStart;
    editor.selectionEnd = previousState.selectionEnd;

    lastSavedState = {
        text: previousState.text,
        selectionStart: previousState.selectionStart,
        selectionEnd: previousState.selectionEnd
    };

    syncHighlight();
    updateLineNumbers();
    updateCursorPosition();

    isUndoRedo = false;
}


function redoEdit() {
    const editor = document.getElementById("code-editor");
    if (!editor || redoStack.length === 0) {
        return;
    }

    isUndoRedo = true;

    undoStack.push({
        text: editor.value,
        selectionStart: editor.selectionStart,
        selectionEnd: editor.selectionEnd
    });

    const nextState = redoStack.pop();
    editor.value = nextState.text;
    editor.selectionStart = nextState.selectionStart;
    editor.selectionEnd = nextState.selectionEnd;

    lastSavedState = {
        text: nextState.text,
        selectionStart: nextState.selectionStart,
        selectionEnd: nextState.selectionEnd
    };

    syncHighlight();
    updateLineNumbers();
    updateCursorPosition();

    isUndoRedo = false;
}


function saveUndoState() {
    const editor = document.getElementById("code-editor");
    if (!editor) return;

    const currentText = editor.value;
    const currentStart = editor.selectionStart;
    const currentEnd = editor.selectionEnd;

    if (lastSavedState && lastSavedState.text === currentText) {
        lastSavedState.selectionStart = currentStart;
        lastSavedState.selectionEnd = currentEnd;
        return;
    }

    if (lastSavedState) {
        undoStack.push({
            text: lastSavedState.text,
            selectionStart: lastSavedState.selectionStart,
            selectionEnd: lastSavedState.selectionEnd
        });
        redoStack = [];
    }

    lastSavedState = {
        text: currentText,
        selectionStart: currentStart,
        selectionEnd: currentEnd
    };
}

document.getElementById("open-folder-button").addEventListener("click", openFolder);




document.addEventListener("keydown", async (event) => {
    if (
        (event.ctrlKey || event.metaKey) &&
        event.key.toLowerCase() === "s"
    ) {
        event.preventDefault();

        if (!currentFile) {
            return;
        }

        try {
            await invoke("save_document", {
                path: currentFile.path
            });

            currentFileContent =
                document.getElementById("code-editor").value;


            const tab = openTabs.find(
                tab => tab.path === currentFile.path
            );

            if (tab) {
                tab.modified = false;
            }

            renderTabs();

            console.log("Saved:", currentFile.path);

        } catch (error) {
            console.error(
                "Failed to save document:",
                error
            );
        }
    }
});




function handleEnterKey(event) {
    const start = codeEditor.selectionStart;
    const text = codeEditor.value.slice(0, start);
    const currentLine = text.split("\n").pop();
    const indentation = currentLine.match(/^[ \t]*/)?.[0] ?? "";

    event.preventDefault();

    const tabWidth = (typeof settings !== "undefined" && settings.tabSize) ? settings.tabSize : 4;
    const extraIndent = currentLine.trimEnd().endsWith("{") ? " ".repeat(tabWidth) : "";
    const insertion = "\n" + indentation + extraIndent;

    codeEditor.setRangeText(
        insertion,
        start,
        codeEditor.selectionEnd,
        "end"
    );

    updateLineNumbers();
    updateCursorPosition();
    clearTimeout(editorChangeTimer);

    editorChangeTimer = setTimeout(
        updateCurrentDocument,
        150
    );
}


function syncHighlight() {
    const editor = document.getElementById("code-editor");
    const highlight = document.getElementById("code-highlight");
    const highlightInner = document.getElementById("code-highlight-inner");

    if (!editor || !highlight || !highlightInner) return;

    // sync scroll
    highlight.scrollTop = editor.scrollTop;
    highlight.scrollLeft = editor.scrollLeft;

    // Update highlight
    const language = currentFile ? detectLanguageFromPath(currentFile.path) : "text";
    const text = editor.value || "";

    if (typeof hljs !== "undefined" && language !== "text" && hljs.getLanguage(language)) {
        try {
            const result = hljs.highlight(text, { language: language, ignoreIllegals: true });
            highlightInner.innerHTML = result.value + (text.endsWith("\n") ? "\n" : "");
            highlightInner.className = `language-${language} hljs`;
        } catch (e) {
            highlightInner.textContent = text;
            highlightInner.className = "hljs";
        }
    } else {
        highlightInner.textContent = text;
        highlightInner.className = "hljs";
    }

    delete highlightInner.dataset.highlighted;

    updateLineNumbers();
}

function detectLanguageFromPath(path) {
    const ext = path.split(".").pop().toLowerCase();
    const langMap = {
        "rs": "rust",
        "js": "javascript",
        "jsx": "javascript",
        "ts": "typescript",
        "tsx": "typescript",
        "html": "html",
        "css": "css",
        "py": "python",
        "go": "go",
        "json": "json",
        "md": "markdown",
        "toml": "toml",
    };
    return langMap[ext] || "text";
}


// --- Context Menu Code ---

let selectedContextEntry = null;

function showContextMenu(event, entry) {
    event.preventDefault();
    event.stopPropagation();

    selectedContextEntry = entry;
    const menu = document.getElementById("file-context-menu");
    if (!menu) return;

    menu.classList.remove("hidden");
    menu.style.left = `${Math.max(8, Math.min(event.clientX, window.innerWidth - menu.offsetWidth - 8))}px`;
    menu.style.top = `${Math.max(8, Math.min(event.clientY, window.innerHeight - menu.offsetHeight - 8))}px`;
}

function hideContextMenu() {
    const menu = document.getElementById("file-context-menu");
    if (menu) {
        menu.classList.add("hidden");
    }
}

function getParentPath(path) {
    const separator = path.includes("\\") ? "\\" : "/";
    const parts = path.split(separator);
    parts.pop();
    return parts.join(separator);
}

function joinPath(dir, name) {
    const separator = dir.includes("\\") ? "\\" : "/";
    return dir.endsWith(separator) ? `${dir}${name}` : `${dir}${separator}${name}`;
}



document.addEventListener("keydown", (event) => {

    if (!event.ctrlKey) {
        return;
    }


    if (event.key.toLowerCase() == "z") {
        event.preventDefault();

        if (event.shiftKey) {
            redoEdit();
        } else {
            undoEdit();
        }
    }

    if (event.key.toLowerCase() === "y") {
        event.preventDefault();
        redoEdit();
    }
});

// Global click closes context menu
document.addEventListener("click", hideContextMenu);
document.addEventListener("contextmenu", event => {
    if (!event.target.closest(".file-entry")) hideContextMenu();
});
window.addEventListener("blur", hideContextMenu);


const codeEditor = document.getElementById("code-editor");

codeEditor.addEventListener("input", () => {

    if (!isUndoRedo) {
        saveUndoState();
    }


    syncHighlight();
    updateCursorPosition();

    clearTimeout(editorChangeTimer);
    editorChangeTimer = setTimeout(updateCurrentDocument, 150);
});

codeEditor.addEventListener("scroll", () => {
    syncHighlight();
});

codeEditor.addEventListener("keyup", updateCursorPosition);
codeEditor.addEventListener("click", updateCursorPosition);
codeEditor.addEventListener("select", updateCursorPosition);

codeEditor.addEventListener("keydown", (event) => {
    if (event.key === "Tab") {
        event.preventDefault();
        const start = codeEditor.selectionStart;
        const end = codeEditor.selectionEnd;
        const tabWidth = (typeof settings !== "undefined" && settings.tabSize) ? settings.tabSize : 4;
        codeEditor.setRangeText(" ".repeat(tabWidth), start, end, "end");
        syncHighlight();
        updateCursorPosition();
        clearTimeout(editorChangeTimer);
        editorChangeTimer = setTimeout(updateCurrentDocument, 150);
        return;
    }

    if (event.key === "Enter") {
        handleEnterKey(event);
        syncHighlight();
    }
});

// Startup trigger
if (document.readyState === "loading") {
    document.addEventListener("DOMContentLoaded", startDragonIDE);
} else {
    startDragonIDE();
}
