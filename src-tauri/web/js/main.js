const { invoke } = window.__TAURI__.core;

let currentFile = null;
let currentFileContent = "";
let editorChangeTimer = null;
let openTabs = [];


async function startDragonIDE() {

    const name = await invoke("get_ide_name");

    console.log("Rust says: ", name);


    const count = await invoke("document_count");

    console.log("Open Document: ", count);
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
        arrow.textContent = "▶";
        icon.textContent = "📁"

        element.addEventListener("click", () => {
            toggleDirectory(element, entry, arrow);
        });
    } else {
        arrow.textContent = "";
        icon.textContent = "📄";

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

        arrow.textContent = "▼";

        return;
    }

    try {

        arrow.textContent = "▼";

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

        arrow.textContent = "↓";
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

        const editor = document.getElementById("code-editor");

        if (welcome) {
            welcome.style.display = "none";
        }

        if (editor) {
            editor.style.display = "block";
            editor.value = content;
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

        const editor = document.getElementById("code-editor");

        const welcome = document.getElementById("welcome-screen");

        if (welcome) {
            welcome.style.display = "none";
        }

        if (editor) {
            editor.style.display = "block";
            editor.value = content;

            updateLineNumbers();
            updateCursorPosition();

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

            const editor = document.getElementById("code-editor");

            const welcome = document.getElementById("welcome-screen");

            if (editor) {
                editor.style.display = "none";
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

document.getElementById("code-editor").addEventListener("input", () => {
    updateLineNumbers();
    updateCursorPosition();

    clearTimeout(editorChangeTimer);

    editorChangeTimer = setTimeout(
        updateCurrentDocument,
        150
    );
});


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

const codeEditor = document.getElementById("code-editor");

codeEditor.addEventListener(
    "scroll",
    syncEditorScroll
);

codeEditor.addEventListener(
    "keyup",
    updateCursorPosition
);

codeEditor.addEventListener(
    "click",
    updateCursorPosition
);

codeEditor.addEventListener(
    "select",
    updateCursorPosition
);

codeEditor.addEventListener(
    "keydown",
    (event) => {
        if (event.key !== "Tab") {
            return;
        }

        event.preventDefault();

        const start = codeEditor.selectionStart;
        const end = codeEditor.selectionEnd;

        codeEditor.setRangeText(
            "    ",
            start,
            end,
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
);

startDragonIDE();