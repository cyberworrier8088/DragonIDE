// Auto-Save & Unsaved Indicator Feature
let autoSaveState = {
    isEnabled: true,
    interval: 3000, // 3 seconds default
    unsavedFiles: new Map(), // Map<path, boolean>
    currentSaveTimer: null,
    savedTimeout: null,
    lastSavedContent: {},
    pendingChanges: {}
};

// Load auto-save settings from localStorage
function loadAutoSaveSettings() {
    try {
        const saved = localStorage.getItem("autoSaveSettings");
        if (saved) {
            const settings = JSON.parse(saved);
            autoSaveState.isEnabled = settings.isEnabled !== false;
            autoSaveState.interval = (settings.interval !== undefined ? settings.interval : 3) * 1000;
        } else {
            autoSaveState.isEnabled = true;
            autoSaveState.interval = 3000;
        }
    } catch (e) {
        console.error("Failed to load autoSaveSettings:", e);
        autoSaveState.isEnabled = true;
        autoSaveState.interval = 3000;
    }
    syncAutoSaveUI();
}

// Save auto-save settings to localStorage
function saveAutoSaveSettings() {
    try {
        const settings = {
            isEnabled: autoSaveState.isEnabled,
            interval: Math.round(autoSaveState.interval / 1000)
        };
        localStorage.setItem("autoSaveSettings", JSON.stringify(settings));
    } catch (e) {
        console.error("Failed to save autoSaveSettings:", e);
    }
}

function saveAutoSettings() {
    saveAutoSaveSettings();
}

// Sync UI controls with current autoSaveState
function syncAutoSaveUI() {
    const autoSaveToggle = document.getElementById("auto-save-toggle");
    const intervalSlider = document.getElementById("auto-save-interval");
    const intervalValue = document.getElementById("auto-save-value");

    if (autoSaveToggle) {
        autoSaveToggle.checked = autoSaveState.isEnabled;
    }

    const sec = Math.round(autoSaveState.interval / 1000) || 3;
    if (intervalSlider) {
        intervalSlider.value = sec;
    }
    if (intervalValue) {
        intervalValue.textContent = sec + "s";
    }
}

// Mark file as unsaved
function markAsUnsaved(path) {
    if (!autoSaveState.isEnabled) return;
    if (!path) return;

    autoSaveState.unsavedFiles.set(path, true);
    if (typeof openTabs !== "undefined") {
        const tab = openTabs.find(t => t.path === path);
        if (tab) tab.modified = true;
    }

    updateTabIndicator(path);
    updateTitlebarModified(path, true);

    // Clear existing timer
    if (autoSaveState.currentSaveTimer) {
        clearTimeout(autoSaveState.currentSaveTimer);
    }

    // Set new timer for auto-save
    autoSaveState.currentSaveTimer = setTimeout(() => {
        autoSaveFile(path);
    }, autoSaveState.interval);
}

// Mark file as saved
function markAsSaved(path) {
    if (!path) return;

    autoSaveState.unsavedFiles.set(path, false);
    if (typeof openTabs !== "undefined") {
        const tab = openTabs.find(t => t.path === path);
        if (tab) tab.modified = false;
    }

    updateTabIndicator(path);
    updateTitlebarModified(path, false);
    showSavedIndicator();
}

// Update tab indicator (red dot)
function updateTabIndicator(path) {
    if (!path) return;
    const fileName = path.split(/[\\/]/).pop();
    const tabs = document.querySelectorAll(".tab");

    tabs.forEach(tab => {
        if (tab.textContent.includes(fileName)) {
            const isUnsaved = autoSaveState.unsavedFiles.get(path);
            if (isUnsaved) {
                tab.classList.add("unsaved");
            } else {
                tab.classList.remove("unsaved");
            }
        }
    });
}

// Update titlebar with modified indicator
function updateTitlebarModified(path, isModified) {
    const titlebarCenter = document.getElementById("titlebar-center");
    if (!titlebarCenter) return;

    if (!path) {
        titlebarCenter.innerHTML = "";
        return;
    }

    const fileName = path.split(/[\\/]/).pop();

    if (isModified) {
        titlebarCenter.innerHTML = `<span>${fileName} <span style="color: var(--text-muted);">(modified)</span></span>`;
    } else {
        titlebarCenter.innerHTML = `<span>${fileName}</span>`;
    }
}

// Show saving indicator
function showSavingIndicator() {
    const indicator = document.getElementById("saving-indicator");
    const savedIndicator = document.getElementById("saved-indicator");

    if (indicator) {
        indicator.style.display = "inline-block";
        indicator.textContent = "SAVING...";
    }
    if (savedIndicator) {
        savedIndicator.style.display = "none";
    }
}

// Show saved indicator
function showSavedIndicator() {
    const indicator = document.getElementById("saved-indicator");
    const savingIndicator = document.getElementById("saving-indicator");

    if (savingIndicator) {
        savingIndicator.style.display = "none";
    }

    if (indicator) {
        indicator.style.display = "inline-block";
        indicator.textContent = "✓ SAVED";
        indicator.style.color = "#4caf50";
    }

    if (autoSaveState.savedTimeout) {
        clearTimeout(autoSaveState.savedTimeout);
    }

    autoSaveState.savedTimeout = setTimeout(() => {
        if (indicator) {
            indicator.style.display = "none";
        }
    }, 2000);
}

// Show error indicator
function showErrorIndicator(message) {
    const indicator = document.getElementById("saved-indicator");
    const savingIndicator = document.getElementById("saving-indicator");

    if (savingIndicator) {
        savingIndicator.style.display = "none";
    }

    if (indicator) {
        indicator.style.display = "inline-block";
        indicator.textContent = "✗ " + message;
        indicator.style.color = "#ff6b6b";

        setTimeout(() => {
            indicator.style.display = "none";
        }, 3000);
    }
}

// Auto-save file
async function autoSaveFile(path) {
    if (!path || typeof currentFile === "undefined" || !currentFile || currentFile.path !== path) {
        return;
    }

    const editor = document.getElementById("code-editor");
    if (!editor) return;

    const content = editor.value;

    // Check if content actually changed
    if (autoSaveState.lastSavedContent[path] === content) {
        console.log("No changes to save for:", path);
        markAsSaved(path);
        return;
    }

    showSavingIndicator();

    try {
        const { invoke } = window.__TAURI__.core;
        // First sync document in Rust backend state, then commit to disk
        await invoke("update_document", { path: path, text: content });
        await invoke("save_document", { path: path });

        // Update state
        autoSaveState.lastSavedContent[path] = content;
        if (typeof currentFileContent !== "undefined") {
            currentFileContent = content;
        }
        markAsSaved(path);

        console.log("Auto-saved:", path);
    } catch (error) {
        console.error("Auto-save failed:", error);
        showErrorIndicator("Auto-save failed");
    }
}

// Show unsaved changes warning dialog
function showUnsavedDialog(path, onSave, onDiscard) {
    const fileName = path.split(/[\\/]/).pop();

    // Clean up any existing dialogs
    document.querySelectorAll(".unsaved-dialog, .unsaved-dialog-backdrop").forEach(el => el.remove());

    // Create backdrop
    const backdrop = document.createElement("div");
    backdrop.className = "unsaved-dialog-backdrop";

    // Create dialog
    const dialog = document.createElement("div");
    dialog.className = "unsaved-dialog";
    dialog.innerHTML = `
        <h3 data-i18n="unsavedWarning">Unsaved Changes</h3>
        <p id="unsaved-msg"></p>
        <div class="unsaved-dialog-buttons">
            <button class="cancel-btn" id="unsaved-cancel-btn" data-i18n="cancel">Cancel</button>
            <button class="discard-btn" id="unsaved-discard-btn" data-i18n="discard">Discard</button>
            <button class="save-btn" id="unsaved-save-btn" data-i18n="save">Save</button>
        </div>
    `;

    document.body.appendChild(backdrop);
    document.body.appendChild(dialog);

    // Set message with filename
    const msgElement = dialog.querySelector("#unsaved-msg");
    if (msgElement) {
        if (typeof t === "function") {
            msgElement.textContent = t("unsavedMessage", { fileName }) || `"${fileName}" has unsaved changes. Save before closing?`;
        } else {
            msgElement.textContent = `"${fileName}" has unsaved changes. Save before closing?`;
        }
    }

    if (typeof applyTranslations === "function") {
        applyTranslations();
    }

    const cleanup = () => {
        backdrop.remove();
        dialog.remove();
        document.removeEventListener("keydown", escapeHandler);
    };

    const escapeHandler = (e) => {
        if (e.key === "Escape") {
            cleanup();
        }
    };
    document.addEventListener("keydown", escapeHandler);
    backdrop.addEventListener("click", cleanup);

    dialog.querySelector("#unsaved-save-btn")?.addEventListener("click", () => {
        cleanup();
        if (onSave) onSave();
    });

    dialog.querySelector("#unsaved-discard-btn")?.addEventListener("click", () => {
        cleanup();
        if (onDiscard) onDiscard();
    });

    dialog.querySelector("#unsaved-cancel-btn")?.addEventListener("click", () => {
        cleanup();
    });
}

// Handle file open - initialize auto-save for this file
function initAutoSaveForFile(path) {
    const editor = document.getElementById("code-editor");
    if (editor && editor.value !== undefined) {
        autoSaveState.lastSavedContent[path] = editor.value;
        if (!autoSaveState.unsavedFiles.has(path)) {
            autoSaveState.unsavedFiles.set(path, false);
        }
        updateTabIndicator(path);
        const isUnsaved = autoSaveState.unsavedFiles.get(path);
        updateTitlebarModified(path, !!isUnsaved);
    }
}

// Hook into editor input event & settings
function initAutoSave() {
    loadAutoSaveSettings();

    const editor = document.getElementById("code-editor");
    if (editor) {
        editor.addEventListener("input", () => {
            if (typeof currentFile !== "undefined" && currentFile && autoSaveState.isEnabled) {
                markAsUnsaved(currentFile.path);
            }
        });
    }

    // Settings: Auto-Save Toggle
    const autoSaveToggle = document.getElementById("auto-save-toggle");
    if (autoSaveToggle) {
        autoSaveToggle.checked = autoSaveState.isEnabled;
        autoSaveToggle.addEventListener("change", () => {
            autoSaveState.isEnabled = autoSaveToggle.checked;
            saveAutoSaveSettings();
            console.log("Auto-save:", autoSaveState.isEnabled ? "enabled" : "disabled");

            if (!autoSaveState.isEnabled) {
                if (autoSaveState.currentSaveTimer) {
                    clearTimeout(autoSaveState.currentSaveTimer);
                    autoSaveState.currentSaveTimer = null;
                }
                if (typeof currentFile !== "undefined" && currentFile) {
                    autoSaveState.unsavedFiles.set(currentFile.path, false);
                    updateTabIndicator(currentFile.path);
                    updateTitlebarModified(currentFile.path, false);
                }
            } else {
                if (typeof currentFile !== "undefined" && currentFile && editor) {
                    if (autoSaveState.lastSavedContent[currentFile.path] !== undefined &&
                        editor.value !== autoSaveState.lastSavedContent[currentFile.path]) {
                        markAsUnsaved(currentFile.path);
                    }
                }
            }
        });
    }

    // Settings: Auto-Save Interval
    const intervalSlider = document.getElementById("auto-save-interval");
    const intervalValue = document.getElementById("auto-save-value");

    if (intervalSlider) {
        const savedInterval = Math.round(autoSaveState.interval / 1000);
        intervalSlider.value = savedInterval;
        if (intervalValue) {
            intervalValue.textContent = savedInterval + "s";
        }

        intervalSlider.addEventListener("input", (e) => {
            const sec = parseInt(e.target.value, 10);
            autoSaveState.interval = sec * 1000;
            if (intervalValue) {
                intervalValue.textContent = sec + "s";
            }
            saveAutoSaveSettings();
        });
    }
}

if (document.readyState === "loading") {
    document.addEventListener("DOMContentLoaded", initAutoSave);
} else {
    initAutoSave();
}

// Prevent data loss on window close / unload
window.addEventListener("beforeunload", (e) => {
    let hasUnsaved = false;
    for (let [path, isUnsaved] of autoSaveState.unsavedFiles) {
        if (isUnsaved) {
            hasUnsaved = true;
            break;
        }
    }

    if (hasUnsaved) {
        e.preventDefault();
        e.returnValue = "";
        return "";
    }
});