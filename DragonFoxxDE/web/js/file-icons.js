

const FILE_ICON_FOLDER = "assets/file-icons/";

const FILE_NAME_ICONS = {
    "cargo.toml": "rust.svg",
    "cargo.lock": "rust.svg",
    ".gitignore": "config.svg",
    ".gitattributes": "config.svg",
    "license": "text.svg",
}

// file extensions
const FILE_EXTENSION_ICONS = {
    rs: "rust.svg",
    py: "python.svg",
    js: "javascript.svg",
    mjs: "javascript.svg",
    cjs: "javascript.svg",
    ts: "typescript.svg",
    tsx: "typescript.svg",
    html: "html.svg",
    htm: "html.svg",
    css: "css.svg",
    json: "json.svg",
    md: "markdown.svg",
    png: "image.svg",
    jpg: "image.svg",
    jpeg: "image.svg",
    gif: "image.svg",
    webp: "image.svg",
    bmp: "image.svg",
    ico: "image.svg",
    svg: "image.svg",
    toml: "config.svg",
    yaml: "config.svg",
    yml: "config.svg",
    ini: "config.svg",
    lock: "config.svg",
    sh: "terminal.svg",
    bat: "terminal.svg",
    cmd: "terminal.svg",
    ps1: "terminal.svg",
    txt: "text.svg",
    log: "text.svg"
};


function fileIconUrl(fileName) {
    const name = fileName.toLowerCase();

    if (FILE_EXTENSION_ICONS.hasOwnProperty(name)) {
        return FILE_ICON_FOLDER + FILE_NAME_ICONS[name];
    }

    const extension = name.includes(".") ? name.split(".").pop() : "";
    if (FILE_EXTENSION_ICONS.hasOwnProperty(extension)) {
        return FILE_ICON_FOLDER + FILE_EXTENSION_ICONS[extension];
    }

    return null; // unknown type
}


function applyFileIcon(element, fileName) {
    const url = fileIconUrl(fileName);
    if (!url) return;

    element.classList.add("has-icon");
    element.style.backgroundImage = `url("${url}")`;
}