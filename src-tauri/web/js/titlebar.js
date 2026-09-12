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

document.getElementById("close-btn").addEventListener("click", async () => {
    // Check if any file has unsaved changes before closing window
    if (typeof autoSaveState !== "undefined" && autoSaveState.unsavedFiles) {
        let unsavedPath = null;
        for (let [path, isUnsaved] of autoSaveState.unsavedFiles) {
            if (isUnsaved) {
                unsavedPath = path;
                break;
            }
        }

        if (unsavedPath && typeof showUnsavedDialog === "function") {
            showUnsavedDialog(
                unsavedPath,
                async () => {
                    if (typeof autoSaveFile === "function") {
                        await autoSaveFile(unsavedPath);
                    }
                    await appWindow.close();
                },
                async () => {
                    await appWindow.close();
                }
            );
            return;
        }
    }

    await appWindow.close();
});

document.getElementById("titlebar").addEventListener("dblclick", async (event) => {
    if (!event.target.closest(".titlebar-button")) {
        await toggleMaximize();
    }
});

function updateTitlebarFileName(fileName) {
    const center = document.getElementById("titlebar-center");
    if (!center) return;

    if (fileName) {
        if (typeof autoSaveState !== "undefined" && typeof currentFile !== "undefined" && currentFile && autoSaveState.unsavedFiles?.get(currentFile.path)) {
            center.innerHTML = `<span>${fileName} <span style="color: var(--text-muted);">(modified)</span></span>`;
        } else {
            center.innerHTML = `<span>${fileName}</span>`;
        }
    } else {
        center.innerHTML = "";
    }
}

window.addEventListener("file-opened", (e) => {
    updateTitlebarFileName(e.detail);
});
