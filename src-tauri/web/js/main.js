const { invoke } = window.__TAURI__.core;

async function startDragonIDE() {

    const name = await invoke("get_ide_name");

    console.log("Rust says: ", name);
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


function renderFileTree(files) {

    const tree = document.getElementById("file-tree");

    tree.innerHTML = "";

    for (const file of files) {
        const entry = createFileEntry(file);

        tree.appendChild(entry);
    }
}

document.getElementById("open-folder-button").addEventListener("click", openFolder);

startDragonIDE();