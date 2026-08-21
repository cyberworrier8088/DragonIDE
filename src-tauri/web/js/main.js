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

        const explorerName = document.querySelector(
            ".explorer-section span"
        );

        const explorerEmpty = document.querySelector(
            ".explorer-empty"
        );

        if (explorerName) {
            explorerName.textContent =
                folder.split(/[\\/]/).pop();
        }

        if (explorerEmpty) {
            explorerEmpty.textContent =
                "Loading files...";
        }
    } catch (error) {
        console.error("Failed to open folder: ", error);
    }
}

document.getElementById("open-folder-button").addEventListener("click", openFolder);

startDragonIDE();