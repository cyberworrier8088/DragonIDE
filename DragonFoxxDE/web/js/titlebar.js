const appWindow = window.__TAURI__.window.getCurrentWindow();

async function toggleMaximize() {
    await appWindow.toggleMaximize();
}

document.getElementById("minimize-btn").addEventListener("click", async () => {
    await appWindow.minimize();
});

document.getElementById("maximize-btn").addEventListener("click", async () => {
    await toggleMaximize();
});

document.getElementById("close-btn").addEventListener("click", () => appWindow.close());

let closeAnyway = false;
appWindow.onCloseRequested?.((event) => {

    const unsaved = openTabs.filter(isTabUnsaved);
    if (closeAnyway || unsaved.length === 0) return;
    event.preventDefault();

    const name = unsaved.length === 1 ? unsaved[0].path : `${unsaved.length} files`;
    showUnsavedDialog(name, async () => {
        try {
            await saveAllTabs(unsaved);
        } catch (error) {
            alert("Could not save: " + error);
            return;
        }
        closeAnyway = true;
        appWindow.close();
    }, () => {
        closeAnyway = true;
        appWindow.close();
    });
});


async function saveAllTabs(tabs) {
    const { invoke } = window.__TAURI__.core;
    const editor = document.getElementById("code-editor");

    if (currentFile && !currentFile.isImage) {
        await invoke("update_document", { path: currentFile.path, text: editor.value });
    }
    for (const tab of tabs) {
        await invoke("save_document", { path: tab.path });
    }
}

document.getElementById("titlebar").addEventListener("dblclick", async (event) => {
    if (!event.target.closest(".titlebar-button")) {
        await toggleMaximize();
    }
});

function updateTitlebarFileName(fileName) {
    const center = document.getElementById("titlebar-center");
    if (!center) return;

    const modified = typeof autoSaveState !== "undefined" && typeof currentFile !== "undefined" && currentFile && autoSaveState.unsavedFiles?.get(currentFile.path);
    setTitlebarText(center, fileName, modified);
}

function setTitlebarText(center, fileName, modified) {
    center.replaceChildren();
    if (!fileName) return;

    const name = document.createElement("span");
    name.textContent = fileName;

    if (modified) {
        const tag = document.createElement("span");
        tag.className = "titlebar-modified";
        tag.textContent = " (modified)";
        name.appendChild(tag);
    }

    center.appendChild(name);
}

window.addEventListener("file-opened", (e) => {
    updateTitlebarFileName(e.detail);
});