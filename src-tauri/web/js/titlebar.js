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
    await appWindow.close();
});

document.getElementById("titlebar").addEventListener("dblclick", async (event) => {
    if (!event.target.closest(".titlebar-button")) {
        await toggleMaximize();
    }
});

function updateTitlebarFileName(fileName) {
    const center = document.getElementById("titlebar-center");
    if (fileName) {
        center.textContent = fileName;
    } else {
        center.textContent = "";
    }
}

window.addEventListener("file-opened", (e) => {
    updateTitlebarFileName(e.detail);
});
