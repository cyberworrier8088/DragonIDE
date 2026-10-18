// Image preview: .png .jpg .gif .webp .bmp .ico open as a picture, not as text.

const IMAGE_TYPES = {
    png: "image/png",
    jpg: "image/jpeg",
    jpeg: "image/jpeg",
    gif: "image/gif",
    webp: "image/webp",
    bmp: "image/bmp",
    ico: "image/x-icon"
};

let imagePreviewUrl = null;
let imageRequestId = 0;

function imageExtension(path) {
    const name = path.split(/[\\/]/).pop();
    return name.includes(".") ? name.split(".").pop().toLowerCase() : "";
}

function isImagePath(path) {
    return IMAGE_TYPES.hasOwnProperty(imageExtension(path || ""));
}

// alias so main.js can call either name
const isImageFile = isImagePath;

function formatBytes(size) {
    if (size < 1024) return `${size} B`;
    if (size < 1024 * 1024) return `${(size / 1024).toFixed(1)} KB`;
    return `${(size / 1024 / 1024).toFixed(1)} MB`;
}

async function openImage(entry) {
    if (currentFile?.path === entry.path) return;

    const request = ++imageRequestId;

    try {
        const { invoke } = window.__TAURI__.core;
        const bytes = await invoke("read_image", { path: entry.path });

        // another file was opened while this one was loading
        if (request !== imageRequestId) return;

        // leaving a code file: keep its text, the same way openFile does
        if (currentFile && !currentFile.isImage) {
            clearTimeout(editorChangeTimer);
            await updateCurrentDocument();
        }
        saveCurrentTabHistory();

        currentFile = { name: entry.name, path: entry.path, isImage: true };
        currentFileContent = "";

        showImage(entry, bytes);
        addOrActivateTab(entry);
        window.dispatchEvent(new CustomEvent("file-opened", { detail: entry.name }));
    } catch (error) {
        console.error("Failed to open image:", error);
    }
}

function showImage(entry, bytes) {
    const data = bytes instanceof ArrayBuffer ? bytes : new Uint8Array(bytes);
    const blob = new Blob([data], { type: IMAGE_TYPES[imageExtension(entry.path)] });

    if (imagePreviewUrl) URL.revokeObjectURL(imagePreviewUrl);
    imagePreviewUrl = URL.createObjectURL(blob);

    const welcome = document.getElementById("welcome-screen");
    const editorContainer = document.getElementById("editor-container");
    const preview = document.getElementById("image-preview");
    const img = document.getElementById("image-preview-img");
    const info = document.getElementById("image-preview-info");

    if (welcome) welcome.style.display = "none";
    if (editorContainer) editorContainer.style.display = "none";
    preview.style.display = "flex";

    img.classList.remove("actual-size");
    info.textContent = formatBytes(blob.size);

    img.onload = () => {
        info.textContent = `${img.naturalWidth} × ${img.naturalHeight} px · ${formatBytes(blob.size)} · click the image to zoom`;
    };
    img.onerror = () => {
        info.textContent = `This image could not be shown · ${formatBytes(blob.size)}`;
    };
    img.src = imagePreviewUrl;
}

function hideImagePreview() {
    imageRequestId++; // cancel an image that is still loading

    const preview = document.getElementById("image-preview");
    const img = document.getElementById("image-preview-img");

    if (preview) preview.style.display = "none";
    if (img) img.removeAttribute("src");

    if (imagePreviewUrl) {
        URL.revokeObjectURL(imagePreviewUrl);
        imagePreviewUrl = null;
    }
}

function initImagePreview() {
    // click: switch between "fit to window" and real size
    document.getElementById("image-preview-img")?.addEventListener("click", (e) => {
        e.currentTarget.classList.toggle("actual-size");
    });
}

if (document.readyState === "loading") {
    document.addEventListener("DOMContentLoaded", initImagePreview);
} else {
    initImagePreview();
}