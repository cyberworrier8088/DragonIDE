// Ai autocomplete: grey "ghost text" at the cursor. Tab accepts, Esc hides :)))
// the suggestions come from my code AI server docs/API.md, or any provider below

// everything except dragon, gemini and anthropic talks the OpenAI chat format
const AI_PROVIDERS = {
    dragon: { url: "https://imu.hackclub.app/v1/autocomplete" },
    openai: { url: "https://api.openai.com/v1/chat/completions", model: "gpt-4.1-mini" },
    gemini: { url: "https://generativelanguage.googleapis.com/v1beta/models/{model}:generateContent", model: "gemini-2.5-flash" },
    anthropic: { url: "https://api.anthropic.com/v1/messages", model: "claude-haiku-5-5" },
    openrouter: { url: "https://openrouter.ai/api/v1/chat/completions", model: "openai/gpt-4.1-mini" },
    groq: { url: "https://api.groq.com/openai/v1/chat/completions", model: "llama-3.1-8b-instant" },
    deepseek: { url: "https://api.deepseek.com/chat/completions", model: "deepseek-chat" },
    mistral: { url: "https://api.mistral.ai/v1/chat/completions", model: "codestral-latest" },
    custom: { url: "http://localhost:11434/v1/chat/completions", model: "qwen2.5-coder" }
};

const AI_SYSTEM = "You complete code. Reply with only the new code that goes at <CURSOR>, never repeat code before it. No explanation, no markdown.";
const AI_MAX_TOKENS = 32; // we only show one line

let aiTimer = null;
let aiController = null;
let aiSuggestion = "";
let aiCursor = -1; // where the suggestion start

function aiProvider() {
    const provider = localStorage.getItem("ai-provider");
    return provider in AI_PROVIDERS ? provider : "dragon";
}

// dragon keeps the old "ai-key" name so saved keys still work
function aiStoreName(field) {
    const provider = aiProvider();
    return provider === "dragon" ? `ai-${field}` : `ai-${field}-${provider}`;
}

function aiSetting(field) {
    return localStorage.getItem(aiStoreName(field)) || "";
}

function aiEnabled() {
    // custom is usually a local server (Ollama, LM Studio) that needs no key
    return localStorage.getItem("ai-enabled") !== "false" && (!!aiSetting("key") || aiProvider() === "custom");
}

// shortcut: 32 output tokens, so "thinking" models (gpt-5, o-series, gemini pro) can reply empty; use a fast model
function buildAiRequest(provider, key, model, prefix, suffix, filename) {
    const prompt = `File: ${filename}\n${prefix}<CURSOR>${suffix}`;

    if (provider === "dragon") return {
        headers: { Authorization: `Bearer ${key}` },
        body: { prefix, suffix, filename, single_line: true, max_tokens: AI_MAX_TOKENS },
        text: data => data.completion
    };

    if (provider === "gemini") return {
        headers: { "x-goog-api-key": key },
        body: {
            systemInstruction: { parts: [{ text: AI_SYSTEM }] },
            contents: [{ parts: [{ text: prompt }] }],
            generationConfig: { maxOutputTokens: AI_MAX_TOKENS, thinkingConfig: { thinkingBudget: 0 } }
        },
        text: data => data.candidates?.[0]?.content?.parts?.[0]?.text
    };

    if (provider === "anthropic") return {
        headers: { "x-api-key": key, "anthropic-version": "2023-06-01", "anthropic-dangerous-direct-browser-access": "true" },
        body: { model, max_tokens: AI_MAX_TOKENS, system: AI_SYSTEM, messages: [{ role: "user", content: prompt }] },
        text: data => data.content?.[0]?.text
    };

    // openrouter/auto and /free can pick a thinking model, which spends all 32 tokens thinking and answers empty
    const noThinking = provider === "openrouter" ? { reasoning: { enabled: false } } : {};

    return {
        headers: { Authorization: `Bearer ${key}` },
        body: { model, max_tokens: AI_MAX_TOKENS, ...noThinking, messages: [{ role: "system", content: AI_SYSTEM }, { role: "user", content: prompt }] },
        text: data => data.choices?.[0]?.message?.content
    };
}

// one request to the chosen provider: the reply text, or throws with the API's error message
async function callAi(prefix, suffix, filename, signal) {
    const provider = aiProvider();
    const model = aiSetting("model") || AI_PROVIDERS[provider].model;
    const url = (aiSetting("url") || AI_PROVIDERS[provider].url).replace("{model}", model);
    const request = buildAiRequest(provider, aiSetting("key"), model, prefix, suffix, filename);

    const response = await fetch(url, {
        method: "POST",
        signal,
        headers: { ...request.headers, "Content-Type": "application/json" },
        body: JSON.stringify(request.body)
    }).catch(error => {
        throw error.name === "AbortError" ? error : new Error("Can't reach the AI server: " + error.message);
    });

    const data = await response.json().catch(() => ({}));
    if (!response.ok) throw new Error(data.error?.message || `Error ${response.status}`);
    return data.cancelled ? "" : request.text(data) || "";
}

// chat models wrap code in ``` and repeat the indent or the start of the line: keep only the new part of the first line
function cleanCompletion(text, line) {
    text = text.replace(/^\s*```[\w+-]*\n?/, "");
    if (!line.trim()) text = text.replace(/^\n+/, ""); // cursor on a fresh line: the reply is that line

    if (text.startsWith(line)) text = text.slice(line.length);
    else if (line.trim() && text.trimStart().startsWith(line.trim())) text = text.trimStart().slice(line.trim().length);

    return text.split("\n")[0];
}

function setAiStatus(text, title = "") {
    const status = document.getElementById("ai-status");

    if (!status) return;
    status.textContent = text;
    status.title = title;
    status.classList.toggle("thinking", text.endsWith("..."));
}

function updateAiStatus() {
    if (aiEnabled()) setAiStatus("✦ AI", `AI autocomplete is on (${aiProvider()})`);
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

function showSuggestion(text, cursor, fresh = false) {

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
    hint.className = fresh ? "ghost-text ghost-new" : "ghost-text"; // fade in only a new suggestion, not each typed letter
    hint.textContent = text;

    // the ) or quote after the cursor is redrawn after the hint, so the hint pushes it right instead of covering it
    const lineEnd = editor.value.indexOf("\n", cursor);
    const restOfLine = document.createElement("span");
    restOfLine.className = "ghost-rest";
    restOfLine.textContent = editor.value.slice(cursor, lineEnd === -1 ? undefined : lineEnd);

    const after = document.createElement("span");
    after.textContent = lineEnd === -1 ? "" : editor.value.slice(lineEnd);

    ghost.replaceChildren(before, hint, restOfLine, after);
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
    aiTimer = setTimeout(fetchSuggestion, 500); // wait for a real pause: fewer requests, fewer tokens
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

    // auto-close brackets puts ) ] } or a quote after the cursor, that still counts as the end of the line
    if (!/^[\s)\]}"'`;,]*$/.test(restOfLine)) return;

    aiController = new AbortController();
    setAiStatus("✦ Ai...", "Thinking");

    try {
        // small context = fewer tokens per request
        const text = await callAi(code.slice(Math.max(0, cursor - 1000), cursor), code.slice(cursor, cursor + 200), currentFile.name, aiController.signal);
        if (text.trim()) updateAiStatus();
        else setAiStatus("⚠ AI", "The model answered empty. Pick a fast model in Settings, like google/gemini-2.0-flash-001");

        if (editor.value !== code || editor.selectionStart !== cursor) return;

        const line = code.slice(code.lastIndexOf("\n", cursor - 1) + 1, cursor);
        let suggestion = cleanCompletion(text, line);
        // the model often writes the ) or quote that auto-close already put after the cursor
        const tail = restOfLine.trim();
        if (tail && suggestion.endsWith(tail)) suggestion = suggestion.slice(0, -tail.length);
        showSuggestion(suggestion, cursor, true);
    } catch (error) {
        if (error.name === "AbortError") return;
        setAiStatus("⚠ AI", error.message);
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

// show the saved key/model/url of the chosen provider
function loadAiFields() {
    const select = document.getElementById("ai-provider");
    if (!select) return;

    const { url, model } = AI_PROVIDERS[aiProvider()];
    const modelInput = document.getElementById("ai-model-input");
    const urlInput = document.getElementById("ai-url-input");

    select.value = aiProvider();
    document.getElementById("ai-key-input").value = aiSetting("key");
    modelInput.value = aiSetting("model");
    modelInput.placeholder = `Model (${model})`;
    modelInput.hidden = !model;
    urlInput.value = aiSetting("url");
    urlInput.placeholder = url;
    document.getElementById("ai-test-result").textContent = "";
    updateAiStatus();
}

function saveAiFields() {
    for (const field of ["key", "model", "url"]) {
        localStorage.setItem(aiStoreName(field), document.getElementById(`ai-${field}-input`).value.trim());
    }
    updateAiStatus();
}

// Done: save, then one tiny request so a wrong key, model or url shows right here
async function testAi() {
    const result = document.getElementById("ai-test-result");
    saveAiFields();
    result.textContent = "Saved. Testing...";

    try {
        const text = cleanCompletion(await callAi("def add(a, b):\n    return ", "\n", "test.py"), "    return ");
        result.textContent = text ? `✓ Works! It answered: ${text}` : "✓ Saved, but the reply was empty. Try a faster model.";
    } catch (error) {
        result.textContent = `✗ ${error.message}`;
    }
}

function initAiComplete() {
    const editor = document.getElementById("code-editor");
    const toggle = document.getElementById("ai-enabled-toggle");
    const select = document.getElementById("ai-provider");

    if (!editor) return;

    if (toggle) {
        toggle.checked = localStorage.getItem("ai-enabled") !== "false";

        toggle.addEventListener("change", () => {
            localStorage.setItem("ai-enabled", toggle.checked);
            clearSuggestion();
            updateAiStatus();
        });
    }

    if (select) {
        select.addEventListener("change", () => {
            localStorage.setItem("ai-provider", select.value);
            clearSuggestion();
            loadAiFields();
        });

        for (const field of ["key", "model", "url"]) {
            document.getElementById(`ai-${field}-input`).addEventListener("change", saveAiFields);
        }
        document.getElementById("ai-done-btn").addEventListener("click", testAi);

        loadAiFields();
    }

    editor.addEventListener("input", onAiInput);
    editor.addEventListener("scroll", syncGhostScroll);
    editor.addEventListener("click", clearSuggestion);
    editor.addEventListener("blur", clearSuggestion);
    document.addEventListener("keydown", onAiKeydown, true);

    updateAiStatus();
}

document.addEventListener("DOMContentLoaded", initAiComplete);
