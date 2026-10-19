let terminalRoot = null;
let terminalCwd = null;
let terminalRunning = false;
let terminalStopRequested = false;
let terminalHistory = [];
let terminalHistoryIndex = 0;
let terminalQueue = [];
let terminalFlushTimer = null;

const TERMINAL_MAX_LINES = 5000;

function terminalCurrentDir() {

    // a different folder was opened: start again from its rooot
    if (window.currentWorkspace != terminalRoot) {
        terminalRoot = window.currentWorkspace;
        terminalCwd = terminalRoot;
    }
    return terminalCwd || ".";
}


function terminalFolderName(dir) {
    return dir.split(/[\\/]/).filter(Boolean).pop() || dir;
}

function updateTerminalPrompt() {
    const prompt = document.querySelector(".terminal-prompt");
    if (!prompt) return;

    const dir = terminalCurrentDir();
    prompt.textContent = `${terminalFolderName(dir)} $`;
    prompt.title = dir;
}

function writeTerminal(text, type = "") {
    terminalQueue.push({ text, type });

    // add lines in small batches, so fast output doesn't freeze the IDE :)
    if (!terminalFlushTimer) {
        terminalFlushTimer = setTimeout(flushTerminal, 30);
    }
}

function flushTerminal() {

    terminalFlushTimer = null;

    const output = document.getElementById("terminal-output");
    if (!output) return;

    const atBottom = output.scrollHeight - output.scrollTop - output.clientHeight < 30;
    const fragment = document.createDocumentFragment();

    for (const { text, type } of terminalQueue) {

        const line = document.createElement("span");
        if (type) line.className = `terminal-${type}`;
        line.textContent = text + "\n";
        fragment.appendChild(line);
    }

    terminalQueue = [];
    output.appendChild(fragment);

    while (output.childNodes.length > TERMINAL_MAX_LINES) {
        output.removeChild(output.firstChild);
    }

    if (atBottom) output.scrollTop = output.scrollHeight;

}

function clearTerminal() {
    terminalQueue = [];

    const output = document.getElementById("terminal-output");
    if (output) output.textContent = "";
}

function setTerminalRunning(running) {
    terminalRunning = running;

    const stopBtn = document.getElementById("terminal-stop-btn");

    if (stopBtn) stopBtn.style.display = running ? "inline-block" : "none";

    const input = document.getElementById("terminal-input");
    if (input) input.placeholder = running ? "Running... (Ctrl+C to stop)" : "";
}

async function changeTerminalDir(dir, target) {

    // "cd" alone goes back to the project folder
    if (!target) {
        terminalCwd = terminalRoot;
        return;
    }

    try {
        const { invoke } = window.__TAURI__.core;
        terminalCwd = await invoke("terminal_change_dir", {
            currentDir: dir,
            target: target
        })
    } catch (error) {
        writeTerminal(String(error), "error");
    }
}

async function runTerminalCommand(commandText) {
    const { invoke, Channel } = window.__TAURI__.core;
    const dir = terminalCurrentDir();

    writeTerminal(`${terminalFolderName(dir)} $ ${commandText}`, "command");
    if (!commandText) return;

    if (terminalHistory[terminalHistory.length - 1] !== commandText) {

        terminalHistory.push(commandText);
    }
    terminalHistoryIndex = terminalHistory.length;

    if (/^(clear|cls)$/i.test(commandText)) {
        clearTerminal();
        return;
    }

    // the shell forgets "cd" after each command, so we keep the folder ourselves
    const cd = commandText.match(/^cd(?:\s+(.*))?$/i);

    if (cd && !/[&|;]/.test(commandText)) {
        const target = (cd[1] || "").trim().replace(/^\/d\s+/i, "");

        await changeTerminalDir(dir, target);
        updateTerminalPrompt();
        return;
    }

    setTerminalRunning(true);
    terminalStopRequested = false;
    const startTime = Date.now();

    const onEvent = new Channel();
    onEvent.onmessage = (message) => {
        if (message.kind === "line") {
            writeTerminal(message.text, message.error ? "error" : "");
        } else if (message.kind === "done") {
            const ms = Date.now() - startTime;

            if (terminalStopRequested) {
                writeTerminal(`■ Stopped after ${ms}ms`, "warning");
            } else if (message.code !== 0) {
                writeTerminal(`Exit code ${message.code}`, "error");
            }
            setTerminalRunning(false);

            // the command may have made, rename or deleted file
            refreshWorkspace().catch(error => console.warn("Explorer refresh failed:", error));
        }
    };

    try {
        await invoke("execute_terminal_command", { command: commandText, currentDir: dir, onEvent: onEvent });
    } catch (error) {
        writeTerminal(`Error: ${error}`, "error");
        setTerminalRunning(false);
    }
}

async function stopTerminalCommand() {

    if (!terminalRunning) return;
    terminalStopRequested = true;

    try {
        const { invoke } = window.__TAURI__.core;
        await invoke("stop_terminal_command");
    } catch (error) {
        writeTerminal(`stop failed: ${error}`, "error");
    }
}

function initTerminal() {
    const input = document.getElementById("terminal-input");
    if (!input) return;

    updateTerminalPrompt();

    input.addEventListener("keydown", (e) => {
        if (e.key === "Enter") {
            e.preventDefault();

            if (terminalRunning) {
                writeTerminal("A command is still running. Press Ctrl+c or Stop first", "warning");
                return;
            }

            const commandText = input.value.trim();
            input.value = "";
            runTerminalCommand(commandText);
        } else if (e.key === "ArrowUp") {
            e.preventDefault();

            if (terminalHistoryIndex > 0) {
                terminalHistoryIndex--;
                input.value = terminalHistory[terminalHistoryIndex];
            }
        } else if (e.key === "ArrowDown") {
            e.preventDefault();

            if (terminalHistoryIndex < terminalHistory.length - 1) {
                terminalHistoryIndex++;
                input.value = terminalHistory[terminalHistoryIndex];
            } else {
                terminalHistoryIndex = terminalHistory.length;
                input.value = "";
            }
        } else if (e.ctrlKey && e.key.toLowerCase() === "l") {
            e.preventDefault();
            clearTerminal();
        } else if (e.ctrlKey && e.key.toLocaleLowerCase() === "c" && terminalRunning && input.selectionStart === input.selectionEnd) {
            // Ctr+C with nothing selected = stop, like a real terminal
            e.preventDefault();
            stopTerminalCommand();
        }
    });

    document.getElementById("terminal-stop-btn")?.addEventListener("click", stopTerminalCommand);

    document.getElementById("terminal-output")?.addEventListener("click", () => {
        if (!window.getSelection().toString()) input.focus();
    });

    document.querySelector('.panel-tab[data-panel="terminal"]')?.addEventListener("click", updateTerminalPrompt);
}

if (document.readyState === "loading") {
    document.addEventListener("DOMContentLoaded", initTerminal);
} else {
    initTerminal();
}