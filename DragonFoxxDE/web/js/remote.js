// Remote mode: code from a phone, while everthing runs on the laptop
// laptop: Settings -> Remote Access starts a small server and shows a QR code
// phone: the browser has no Tauri, so every invoke() goes to the laptop over WiFi

if (!window.__TAURI__) {
    const params = new URLSearchParams(location.search);
    if (params.has("token")) {
        localStorage.setItem("remote-token", params.get("token"));
        history.replaceState(null, "", location.pathname); // hide the token from the addr bar
    }

    const remoteInvoke = async (command, args = {}) => {
        const response = await fetch(`/api/${command}`, {
            method: "POST",
            headers: {
                Authorization: `Bearer ${localStorage.getItem("remote-token")}`,
                "Content-Type": "application/json"
            },
            body: JSON.stringify(args)
        });

        const data = await response.json();

        if (!response.ok) throw data.error; // like Tauri: the error is the msg from Rust
        return data.result;
    };

    const runTerminal = async ({ onEvent, ...args }) => {
        await remoteInvoke("execute_terminal_command", args);

        while (true) {
            const events = await remoteInvoke("terminal_poll");
            for (const event of events) {
                onEvent.onmessage?.(event);
                if (event.kind === "done") return;
            }

            await new Promise(resolve => setTimeout(resolve, 300));
        }
    };

    const invoke = (command, args) => command === "execute_terminal_command" ? runTerminal(args) : remoteInvoke(command, args);

    const noWindow = {
        minimize: async () => { },
        toggleMaximize: async () => { },
        close: async () => { },
        setTheme: async () => { },
        setEffects: async () => { },
        clearEffects: async () => { }
    };

    window.isRemote = true;
    document.documentElement.classList.add("remote");

    window.__TAURI__ = {
        core: { invoke, Channel: class { onmessage = null; } },
        dialog: { open: async () => null, confirm: async (message) => confirm(message) },
        window: { getCurrentWindow: () => noWindow }
    };
}

function initRemoteSettings() {
    const startButton = document.getElementById("remote-start-btn");
    const stopButton = document.getElementById("remote-stop-btn");
    const info = document.getElementById("remote-info");
    const qr = document.getElementById("remote-qr");
    const url = document.getElementById("remote-url");
    if (!startButton || window.isRemote) return;

    const { invoke } = window.__TAURI__.core;

    startButton.addEventListener("click", async () => {
        if (!window.currentWorkspace) {
            alert("Open a folder first.");
            return;
        }

        try {
            const remote = await invoke("start_remote", {
                workspace: window.currentWorkspace,
                allowRun: document.getElementById("remote-allow-run").checked
            });

            qr.innerHTML = remote.qr_svg; // made by our own rust code, not from the internet
            url.value = remote.url;
            info.style.display = "block";
            startButton.style.display = "none";
            stopButton.style.display = "";
        } catch (error) {
            alert("Could not start remote access: " + error);
        }
    });

    stopButton.addEventListener("click", async () => {
        await invoke("stop_remote");
        qr.innerHTML = "";
        url.value = "";
        info.style.display = "none";
        startButton.style.display = "";
        stopButton.style.display = "none"
    });
}

document.addEventListener("DOMContentLoaded", initRemoteSettings);


function initPhoneLayout() {
    if (!window.isRemote) return;

    document.querySelector('meta[name="viewport"]')?.setAttribute("content", "width=device-width, initial-scale=1, maximum-scale=1");

    const closeMenu = () => document.documentElement.classList.remove("menu-open");

    const menuButton = document.createElement("button");
    menuButton.id = "phone-menu-btn";
    menuButton.className = "titlebar-button";
    menuButton.title = "Files";
    menuButton.textContent = "☰";
    menuButton.addEventListener("click", () => document.documentElement.classList.toggle("menu-open"));
    document.getElementById("titlebar-left")?.prepend(menuButton);

    const saveButton = document.createElement("button");
    saveButton.id = "phone-save-btn";
    saveButton.className = "titlebar-button";
    saveButton.title = "Save";
    saveButton.textContent = "S";
    saveButton.addEventListener("click", () => {
        document.dispatchEvent(new KeyboardEvent("keydown", { key: "s", ctrlKey: true, bubbles: true }));
    });

    document.getElementById("run-btn")?.before(saveButton);

    // tap next to the open menu to close it
    const backdrop = document.createElement("div");
    backdrop.id = "phone-backdrop";
    backdrop.addEventListener("click", closeMenu);
    document.getElementById("app")?.append(backdrop);

    window.addEventListener("file-opened", closeMenu);

    let pressTimer = null;
    let longPressed = false;
    document.addEventListener("touchstart", (event) => {
        longPressed = false;
        if (!event.target.closest("#file-explorer")) return;
        const touch = event.touches[0];
        pressTimer = setTimeout(() => {
            longPressed = true;
            event.target.dispatchEvent(new MouseEvent("contextmenu", {
                bubbles: true,
                clientX: touch.clientX,
                clientY: touch.clientY
            }));
        }, 550);
    }, { passive: true });

    for (const name of ["touchend", "touchmove", "touchcancel", "contextmenu"]) {
        document.addEventListener(name, () => clearTimeout(pressTimer), true);
    }

    document.addEventListener("click", (event) => {
        if (!longPressed) return;
        longPressed = false;
        event.preventDefault();
        event.stopPropagation();
    }, true);
}

document.addEventListener("DOMContentLoaded", initPhoneLayout);