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
        core: { invoke: remoteInvoke, Channel: class { } },
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
                workspace: window.currentWorkspace
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