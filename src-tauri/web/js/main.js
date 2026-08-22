const { invoke } = window.__TAURI__.core;

let currentFile = null;
let currentFileContent = "";
let editorChangeTimer = null;


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
        arrow.textContent = ":>"
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

        arrow.textContent = ":>";

        return;
    }

    try {

        arrow.textContent = "↓";

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

        arrow.textContent = ":>";
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
            editor.focus();
        }


        updateEditorTab(entry.name);

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

        updateEditorTab(currentFile.name, true);
    } catch (error) {

        console.error(
            "Failed to update document:",
            error
        );
    }
}

document.getElementById("open-folder-button").addEventListener("click", openFolder);

document.getElementById("code-editor").addEventListener("input", () => {
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

            updateEditorTab(currentFile.name, false);

            console.log("Saved:", currentFile.path);

        } catch (error) {
            console.error(
                "Failed to save document:",
                error
            );
        }
    }
});

startDragonIDE();