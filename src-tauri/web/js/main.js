const { invoke } = window.__TAURI__.core;

let currentFile = null;
let currentFileContent = "";
let editorChangeTimer = null;
let openTabs = [];


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
}

async function openFolder() {
    try {

        const folder = await invoke("open_folder");

        if (folder === null) {
            console.log("Folder selection cancelled");
            return;
        }

        console.log("Selected workspace: ", folder);

        const explorerName =
            document.getElementById("workspace-name");

        if (explorerName) {
            explorerName.textContent =
                folder.split(/[\\/]/).pop();
        }

        const files = await invoke("read_workspace", {
            path: folder
        });

        console.log("Workspace files: ", files);

        renderFileTree(files);

    } catch (error) {
        console.error("Failed to open folder: ", error);
    }
}


function createFileEntry(entry) {

    const element = document.createElement("div");

    element.classList.add("file-entry");

    if (entry.is_directory) {
        element.classList.add("directory");
    }

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
            modified: false
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

        close.addEventListener("click", (event) => {
            event.stopPropagation();

            closeTab(tab.path);
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

    try {

        const content = await invoke("open_document", {
            path: tab.path
        });

        currentFile = {
            name: tab.name,
            path: tab.path
        };

        currentFileContent = content;

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

function closeTab(path) {
    const index = openTabs.findIndex(
        tab => tab.path === path
    );

    if (index === -1) {
        return;
    }

    const wasActive = currentFile && currentFile.path === path;

    openTabs.splice(index, 1);

    if (wasActive) {
        if (openTabs.length === 0) {
            currentFile = null;
            currentFileContent = "";

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
        } else {
            const nextIndex = Math.min(index, openTabs.length - 1);

            activateTab(openTabs[nextIndex].path);
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
}



function syncEditorScroll() {
    const editor = document.getElementById("code-editor");

    const lineNumbers = document.getElementById("line-numbers");

    if (!editor || !lineNumbers) {
        return;
    }

    lineNumbers.scrollTop = editor.scrollTop;
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


const codeEditor = document.getElementById("code-editor");

codeEditor.addEventListener("input", () => {
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
