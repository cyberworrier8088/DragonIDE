// Ai autocomplete: grey "ghost text" at the cursor. Tab accepts, Esc hides :)))
// the suggestions come from my code AI server API.md

const AI_URL = "https://imu.hackclub.app/v1/autocomplete";

let aiTimer = null;
let aiController = null;
let aiSuggestion = "";
let aiCursor = -1; // where the suggestion start

function aiEnabled() {
    return localStorage.getItem("ai-enabled") !== "false" && !!localStorage.getItem("ai-key");
}

function setAiStatus(text, title = "") {
    const status = document.getElementById("ai-status");

    if (!status) return;
    status.textContent = text;
    status.title = title;
}

function updateAiStatus() {
    if (aiEnabled()) setAiStatus("✦ AI", "AI autocomplete is on");
    else setAiStatus("AI off", "Add your API key in Settings");
}

function clearSuggestion() {
    clearTimeout(aiTimer);
    aiController?.abort();
    aiSuggestion = "";
    aiCursor = -1;
    const ghost = document.getElementById("code-ghost");
    if (ghost) ghost.textContent = "";
}

function showSuggestion(text, cursor) {

    const editor = document.getElementById("code-editor");
    const ghost = document.getElementById("code-ghost");
    if (!text || !ghost) return clearSuggestion();

    aiSuggestion = text;
    aiCursor = cursor;

    // same size and wrapping as the editor
    ghost.style.fontSize = editor.style.fontSize;
    ghost.style.tabSize = editor.style.tabSize;
    ghost.style.whiteSpace = editor.style.whiteSpace;

    const before = document.createElement("span");
    before.textContent = editor.value.slice(0, cursor);

    const hint = document.createElement("span");
    hint.className = "ghost-text";
    hint.textContent = text;

    const after = document.createElement("span");
    after.textContent = editor.value.slice(cursor);

    ghost.replaceChildren(before, hint, after);
    syncGhostScroll();
}

function syncGhostScroll() {
    const editor = document.getElementById("code-editor");
    const ghost = document.getElementById("code-ghost");
    if (!editor || !ghost) return;
    ghost.scrollTop = editor.scrollTop;
    ghost.scrollLeft = editor.scrollLeft;

}

function requestSuggestion() {
    clearSuggestion();
    aiTimer = setTimeout(fetchSuggestion, 300);
}

async function fetchSuggestion() {

    if (!aiEnabled() || !currentFile || currentFile.isImage) return;

    const editor = document.getElementById("code-editor");
    const code = editor.value;
    const cursor = editor.selectionStart;

    if (editor.selectionEnd !== cursor) return; // text is selected
    if (!code.trim()) return;

    const lineEnd = code.indexOf("\n", cursor);
    const restOfLine = code.slice(cursor, lineEnd === -1 ? code.length : lineEnd);

    if (restOfLine.trim() !== "") return;

    aiController = new AbortController();
    setAiStatus("✦ Ai...", "Thinking");

    try {
        const response = await fetch(AI_URL, {
            method: "POST",
            signal: aiController.signal,
            headers: {
                Authorization: `Bearer ${localStorage.getItem("ai-key")}`,
                "Content-Type": "application/json"
            },
            body: JSON.stringify({
                prefix: code.slice(Math.max(0, cursor - 2000), cursor),
                suffix: code.slice(cursor, cursor + 500),
                filename: currentFile.name,
                single_line: true,
                max_tokens: 48
            })
        });

        const data = await response.json();

        if (!response.ok) {
            setAiStatus("⚠ AI", data.error?.message || `Error ${response.status}`);
            return;
        }

        updateAiStatus();

        if (data.cancelled || editor.value !== code || editor.selectionStart !== cursor) return;

        showSuggestion(data.completion.split("\n")[0], cursor);
    } catch (error) {
        if (error.name === "AbortError") return;
        setAiStatus("⚠ AI", "Can't reach the AI server: " + error.message);
    }
}


function acceptSuggestion() {
    const editor = document.getElementById("code-editor");
    const text = aiSuggestion;
    clearSuggestion();

    editor.setRangeText(text, editor.selectionStart, editor.selectionEnd, "end");
    editor.dispatchEvent(new Event("input"));
}

function onAiInput(event) {
    const editor = event.target;
    const cursor = editor.selectionStart;

    // typed the next letter of the suggestion: keep showuing the rest
    if (aiSuggestion && event.inputType === "insertText" && cursor === aiCursor + 1 && editor.value[aiCursor] === aiSuggestion[0]) {
        const rest = aiSuggestion.slice(1);
        if (rest) {
            showSuggestion(rest, cursor);
            return;
        }
    }

    requestSuggestion();
}

function onAiKeydown(event) {
    if (event.target.id !== "code-editor") return;
    if (aiSuggestion && event.key === "Tab" && !event.shiftKey) {
        event.preventDefault();
        event.stopImmediatePropagation();
        acceptSuggestion();
    } else if (aiSuggestion && event.key === "Escape") {

        event.preventDefault();
        clearSuggestion();
    } else if (event.key === "Enter") {
        // main.js adds the new line itself
        requestSuggestion();
    } else if (event.key.startsWith("Arrow") || ["Home", "End", "PageUp", "PageDown"].includes(event.key)) {
        clearSuggestion();
    }
}


function initAiComplete() {
    const editor = document.getElementById("code-editor");
    const toggle = document.getElementById("ai-enabled-toggle");
    const keyInput = document.getElementById("ai-key-input");

    if (!editor) return;

    if (toggle) {
        toggle.checked = localStorage.getItem("ai-enabled") !== "false";
        
        toggle.addEventListener("change", () => {
            localStorage.setItem("ai-enabled", toggle.checked);
            clearSuggestion();
            updateAiStatus();
        });
    }

    if (keyInput) {
        keyInput.value = localStorage.getItem("ai-key") || "";
        keyInput.addEventListener("change", () => {
            localStorage.setItem("ai-key", keyInput.value.trim());
            updateAiStatus();
        });
    }

    editor.addEventListener("input", onAiInput);
    editor.addEventListener("scroll", syncGhostScroll);
    editor.addEventListener("click", clearSuggestion);
    editor.addEventListener("blur", clearSuggestion);
    document.addEventListener("keydown", onAiKeydown, true);

    updateAiStatus();
}

document.addEventListener("DOMContentLoaded", initAiComplete);