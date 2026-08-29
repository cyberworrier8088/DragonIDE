
const SETTINGS_KEY = "dragonide_settings";

const defaultSettings = {
    theme: "dark",
    fontSize: 14,
    tabSize: 4,
    lineWrap: false,
    language: "en"
};

// Load settings from localStorage
function loadSettings() {
    try {
        const saved = localStorage.getItem(SETTINGS_KEY);
        return saved ? { ...defaultSettings, ...JSON.parse(saved) } : { ...defaultSettings };
    } catch (e) {
        console.error("Failed to load settings:", e);
        return { ...defaultSettings };
    }
}

// Save settings to localStorage
function saveSettings(newSettings) {
    try {
        localStorage.setItem(SETTINGS_KEY, JSON.stringify(newSettings));
    } catch (e) {
        console.error("Failed to save settings:", e);
    }
}

// Current settings object
let settings = loadSettings();

function openSettings() {
    const modal = document.getElementById("settings-modal");
    if (modal) {
        modal.classList.remove("hidden");
        updateSettingsUI();
    }
}

function closeSettings() {
    const modal = document.getElementById("settings-modal");
    if (modal) {
        modal.classList.add("hidden");
    }
}

function updateSettingsUI() {
    document.querySelectorAll(".theme-option").forEach(btn => {
        btn.classList.toggle("active", btn.dataset.theme === settings.theme);
    });

    const fontSlider = document.getElementById("font-size-slider");
    const fontValue = document.getElementById("font-size-value");
    if (fontSlider) fontSlider.value = settings.fontSize;
    if (fontValue) fontValue.textContent = settings.fontSize + "px";

    document.querySelectorAll(".tab-option").forEach(btn => {
        btn.classList.toggle("active", parseInt(btn.dataset.tabSize, 10) === settings.tabSize);
    });

    document.querySelectorAll(".lang-option").forEach(btn => {
        btn.classList.toggle("active", btn.dataset.lang === settings.language);
    });

    const lineWrapToggle = document.getElementById("line-wrap-toggle");
    if (lineWrapToggle) {
        lineWrapToggle.checked = !!settings.lineWrap;
    }
}

function applyTheme(themeName) {
    const themeMap = {
        "dark": "css/themes/dark.css",
        "light": "css/themes/light.css",
        "nord": "css/themes/nord.css",
        "monokai": "css/themes/monokai.css",
        "glassmorphism": "css/themes/glassmorphism.css"
    };

    if (!themeMap[themeName]) {
        console.error("Theme not found:", themeName);
        return;
    }

    let themeLink = document.getElementById("highlight-theme");
    if (!themeLink) {
        themeLink = document.createElement("link");
        themeLink.id = "highlight-theme";
        themeLink.rel = "stylesheet";
        document.head.appendChild(themeLink);
    }

    themeLink.href = themeMap[themeName];
    settings.theme = themeName;
    saveSettings(settings);
    console.log("Theme applied:", themeName);
}

function applyFontSize(size) {
    const editor = document.getElementById("code-editor");
    const highlight = document.getElementById("code-highlight");
    const lineNumbers = document.getElementById("line-numbers");

    if (editor) editor.style.fontSize = size + "px";
    if (highlight) highlight.style.fontSize = size + "px";
    if (lineNumbers) lineNumbers.style.fontSize = size + "px";

    settings.fontSize = size;
    saveSettings(settings);
}

function applyTabSize(size) {
    const editor = document.getElementById("code-editor");
    const highlight = document.getElementById("code-highlight");

    if (editor) editor.style.tabSize = size;
    if (highlight) highlight.style.tabSize = size;

    settings.tabSize = size;
    saveSettings(settings);
}

function applyLineWrap(enabled) {
    const editor = document.getElementById("code-editor");
    const highlight = document.getElementById("code-highlight");

    if (editor) {
        editor.wrap = enabled ? "soft" : "off";
        editor.style.whiteSpace = enabled ? "pre-wrap" : "pre";
    }
    if (highlight) highlight.style.whiteSpace = enabled ? "pre-wrap" : "pre";

    settings.lineWrap = enabled;
    saveSettings(settings);
}

function applyAllSettings() {
    applyTheme(settings.theme);
    applyFontSize(settings.fontSize);
    applyTabSize(settings.tabSize);
    applyLineWrap(settings.lineWrap);
}

// Bind event listeners when DOM is loaded
document.addEventListener("DOMContentLoaded", () => {
    applyAllSettings();
    setLanguage(settings.language).then(updateSettingsUI);

    const settingsBtn = document.getElementById("settings-button");
    if (settingsBtn) {
        settingsBtn.addEventListener("click", openSettings);
    }

    const closeBtn = document.getElementById("close-settings");
    if (closeBtn) {
        closeBtn.addEventListener("click", closeSettings);
    }

    const backdrop = document.querySelector(".modal-backdrop");
    if (backdrop) {
        backdrop.addEventListener("click", closeSettings);
    }

    document.querySelectorAll(".theme-option").forEach(btn => {
        btn.addEventListener("click", () => {
            applyTheme(btn.dataset.theme);
            updateSettingsUI();
            if (typeof syncHighlight === "function") {
                syncHighlight();
            }
        });
    });

    const fontSlider = document.getElementById("font-size-slider");
    if (fontSlider) {
        fontSlider.addEventListener("input", (e) => {
            const size = parseInt(e.target.value, 10);
            const fontValue = document.getElementById("font-size-value");
            if (fontValue) fontValue.textContent = size + "px";
            applyFontSize(size);
        });
    }

    document.querySelectorAll(".tab-option").forEach(btn => {
        btn.addEventListener("click", () => {
            applyTabSize(parseInt(btn.dataset.tabSize, 10));
            updateSettingsUI();
        });
    });

    document.querySelectorAll(".lang-option").forEach(btn => {
        btn.addEventListener("click", async () => {
            await setLanguage(btn.dataset.lang);
            updateSettingsUI();
        });
    });

    const lineWrapToggle = document.getElementById("line-wrap-toggle");
    if (lineWrapToggle) {
        lineWrapToggle.addEventListener("change", (e) => {
            applyLineWrap(e.target.checked);
        });
    }

    document.addEventListener("keydown", (e) => {
        if (e.key === "Escape") {
            closeSettings();
        }
    });
});
