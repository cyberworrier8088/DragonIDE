let executionState = {
    isRunning: false,
    currentFile: null,
    output: [],
    startTime: null
};

function detectExecutionLanguage(filePath) {
    if (!filePath) return null;

    const ext = filePath.split('.').pop().toLowerCase();

    const languageMap = {
        "rs": "rust",
        "js": "javascript",
        "py": "python"
    }

    return languageMap[ext] || null;
}

function showOutputPanel() {
    const panel = document.getElementById("panel-content");
    const noOutput = document.getElementById("no-output");
    const outputContent = document.getElementById("output-content");

    if (noOutput) noOutput.style.display = "none";
    if (outputContent) outputContent.style.display = "block";
}

function clearOutput() {
    executionState.output = [];

    const outputContent = document.getElementById("output-content");

    if (outputContent) {
        outputContent.innerHTML = "";
    }

    const noOutput = document.getElementById("no-output");
    if (noOutput) noOutput.style.display = "block";
    if (outputContent) outputContent.style.display = "none";
}

function addOutputLine(text, type = 'normal') {
    executionState.output.push({ text, type });

    const outputContent = document.getElementById("output-content");

    if (!outputContent) return;

    const line = document.createElement("div");

    line.className = `output-line ${type}`;
    line.textContent = text;
    outputContent.appendChild(line);

    // auto-scroll to bottom
    outputContent.scrollTop = outputContent.scrollHeight;
}

function showExecutionStatus(isRunning) {

    const statusDiv = document.getElementById("execution-status");
    const statusText = document.getElementById("status-text");
    const runBtn = document.getElementById("run-btn");

    if (!statusDiv || !statusText) return;

    if (isRunning) {

        statusDiv.style.display = "flex";
        statusText.textContent = "RUNNING....";
        runBtn.classList.add("running");
    } else {
        statusDiv.style.display = "none";
        runBtn.classList.remove("running");
    }
}


function updateExecutionTime(ms) {
    const timeSpan = document.getElementById("execution-time");

    if (timeSpan) {
        timeSpan.textContent = `${ms}ms`;
    }
}


async function runCode() {
    if (!currentFile) {
        console.warn("No file selected");
        addOutputLine("Error: No file selected", "error");
        return;
    }

    if (executionState.isRunning) {
        console.warn("Already running");
        return;
    }

    const language = detectExecutionLanguage(currentFile.path);

    if (!language) {
        addOutputLine("Error: Unsupported file type. Supported: .rs, .js, .py ", "error");
        return;
    }


    clearOutput();
    showOutputPanel();
    showExecutionStatus(true);
    executionState.isRunning = true;
    executionState.startTime = Date.now();

    const editor = document.getElementById("code-editor");
    const code = editor ? editor.value : "";

    try {
        const { invoke } = window.__TAURI__.core;

        try {
            await invoke("update_document", { 
                path: currentFile.path, 
                text: code 
            });
            await invoke("save_document", {
                path: currentFile.path
            });
        } catch (saveError) {
            console.warn("Could not save before execution:", saveError);
        }

        const result = await invoke("execute_code", {
            filePath: currentFile.path,
            language: language,
            code: code
        });

        const executionTime = Date.now() - executionState.startTime;

        if (result.stdout) {
            result.stdout.split('\n').forEach(line => {
                if (line.trim()) {
                    addOutputLine(line, 'normal');
                }
            });
        }

        if (result.stderr) {
            result.stderr.split('\n').forEach(line => {
                if (line.trim()) {
                    addOutputLine(line, 'error');
                }
            });
        }

        if (result.success) {
            addOutputLine(`\nExecution completed in ${executionTime}ms`, "success");
            updateExecutionTime(executionTime);
        } else {
            addOutputLine(`\n✗ Execution failed`, 'error');
            updateExecutionTime(executionTime);
        }

    } catch (error) {
        const executionTime = Date.now() - executionState.startTime;
        console.error("Execution error:", error);
        
        let errorMsg = typeof error === 'string' ? error : (error && error.message ? error.message : JSON.stringify(error));
        addOutputLine(`Error: ${errorMsg}`, 'error');
        addOutputLine(`Execution failed after ${executionTime}ms`, 'error');
        updateExecutionTime(executionTime);
    } finally {
        executionState.isRunning = false;
        showExecutionStatus(false);
    }
}


async function copyOutput() {

    const text = executionState.output.map(line => line.text).join('\n');

    try {
        await navigator.clipboard.writeText(text);
        console.log("Output copied to clipboard!");

    } catch (error) {
        console.error("Failed to copy: ", error);
    }
}

document.addEventListener("DOMContentLoaded", () => {

    const runBtn = document.getElementById("run-btn");

    if (runBtn) {
        runBtn.addEventListener("click", runCode);
    }

    const clearBtn = document.getElementById("clear-output-btn");
    if (clearBtn) {
        clearBtn.addEventListener("click", clearOutput);
    }

    const copyBtn = document.getElementById("copy-output-btn");
    if (copyBtn) {
        copyBtn.addEventListener("click", copyOutput);
    }

    document.addEventListener("keydown", (e) => {
        if ((e.ctrlKey || e.metaKey) && e.altKey && e.key === "r") {
            e.preventDefault();
            runCode();
        }
    });
});


window.runCode = runCode;
window.clearOutput = clearOutput;