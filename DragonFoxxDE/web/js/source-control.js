

let gitStatus = null;
let gitBusy = false;
let gitRequestId = 0;

const GIT_STATUS_NAMES = {
    M: "Modified",
    A: "Added",
    D: "Deleted",
    R: "Renamed",
    C: "Copied",
    U: "Untracked",
    "!": "Conflict"
};

function gitInvoke(command, args = {}) {
    const { invoke } = window.__TAURI__.core;
    return invoke(command, { root: window.currentWorkspace, ...args });
}

function showGitNote(text, isError = false) {
    const note = document.getElementById("git-note");
    if (!note) return;

    note.textContent = text || "";
    note.classList.toggle("error", isError);
}

async function refreshGit() {
    const requestId = ++gitRequestId;

    if (!window.currentWorkspace) {
        gitStatus = null;
        renderGit(null, "Open a folder first.");
        return;
    }

    try {
        const status = await gitInvoke("git_status");

        if (requestId !== gitRequestId) return; // a newer refresh already started

        gitStatus = status;
        renderGit(status);
    } catch (error) {
        if (requestId !== gitRequestId) return;
        showGitNote(String(error), true);
    }
}

function renderGit(status, hint) {
    const noRepo = document.getElementById("git-no-repo");
    const repo = document.getElementById("git-repo");
    const badge = document.getElementById("git-badge");
    if (!status) {
        if (repo) repo.style.display = "none";
        if (noRepo) {
            noRepo.style.display = "block";
            noRepo.querySelector(".git-hint").textContent = hint || "This folder is not a Git repository";
            document.getElementById("git-init-btn").style.display = hint ? "none" : "";
        }

        if (badge) badge.style.display = "none";
        return;
    }

    if (noRepo) noRepo.style.display = "none";
    if (repo) repo.style.display = "flex";

    let branch = `⎇ ${status.branch}`;
    if (status.ahead) branch += ` ↑${status.ahead}`;
    if (status.behind) branch += `  ↓${status.behind}`;
    document.getElementById("git-branch").textContent = branch;

    const staged = status.files.filter(file => file.staged);
    const changes = status.files.filter(file => !file.staged);

    fillGitList("git-staged-list", staged, "unstage");
    fillGitList("git-changes-list", changes, "stage");

    document.getElementById("git-staged-count").textContent = staged.length ? staged.length : "";
    document.getElementById("git-changes-count").textContent = changes.length ? changes.length : "";

    const changedFiles = new Set(status.files.map(file => file.path)).size;

    // number on the acyivity bar button, like VS code
    if (badge) {
        badge.textContent = changedFiles > 99 ? "99+" : String(changedFiles);

        badge.style.display = changedFiles ? "" : "none";
    }
}

function fillGitList(listId, files, action) {
    const list = document.getElementById(listId);

    if (!list) return;

    list.innerHTML = "";
    const fragment = document.createDocumentFragment();

    for (const file of files) {
        const isFolder = file.path.endsWith("/"); // a repo inside the project shows as one folder row
        const parts = file.path.replace(/\/$/, "").split("/");
        const name = parts.pop();

        const row = document.createElement("div");
        row.className = "git-file";
        row.title = `${file.path} — ${GIT_STATUS_NAMES[file.status] || file.status}`

        const icon = document.createElement("span");
        icon.className = "tab-icon";
        if (typeof applyFileIcon === "function") applyFileIcon(icon, name);

        const nameSpan = document.createElement("span");
        nameSpan.className = "git-file-name";
        nameSpan.textContent = name;

        const folder = document.createElement("span");
        folder.className = "git-file-folder";
        folder.textContent = parts.join("/");

        const button = document.createElement("button");
        button.className = "git-icon-btn";
        button.textContent = action === "stage" ? "+" : "-";
        button.title = action === "stage" ? "Stage" : "Unstage";
        button.addEventListener("click", (e) => {
            e.stopPropagation();
            runGitAction(() => gitInvoke(action === "stage" ? "git_stage" : "git_unstage", { paths: [file.path] }));
        });

        const letter = document.createElement("span");
        letter.className = `git-status git-status-${file.status === "!" ? "conflict" : file.status}`;
        letter.textContent = file.status;

        row.append(icon, nameSpan, folder, button, letter);

        if (file.status !== "D" && !isFolder) {
            row.addEventListener("click", () => {
                openFile({ name: name, path: file.full_path, is_directory: false });
            });
        }

        fragment.appendChild(row);
    }

    list.appendChild(fragment);
}

async function runGitAction(action, startNote = "") {
    if (gitBusy) return;

    gitBusy = true;
    showGitNote(startNote); // clear the last message
    document.getElementById("git-view")?.classList.add("busy"); // spins the refresh icon
    document.querySelectorAll("#git-view button").forEach(button => button.disabled = true);

    try {
        await action();
    } catch (error) {
        showGitNote(String(error), true);
    } finally {
        gitBusy = false;
        document.querySelectorAll("#git-view button").forEach(button => button.disabled = false);
        await refreshGit();
        document.getElementById("git-view")?.classList.remove("busy");
    }
}

function commitGit() {
    const box = document.getElementById("git-message");
    const message = box.value.trim();

    if (!message) {
        showGitNote("Write a commit message first.", true);
        box.focus();
        return;
    }

    // never stage for the user: untracked secrets and conflict markers would be committed
    if (!gitStatus || !gitStatus.files.some(file => file.staged)) {
        showGitNote("Nothing is staged. Use + to stage files.", true);
        return;
    }

    runGitAction(async () => {
        const result = await gitInvoke("git_commit", { message: message });

        box.value = "";
        showGitNote(result.split("\n")[0]);
    });
}


function pushOrPullGit(command, doneText) {
    const startNote = command === "git_push" ? "Pushing..." : "Pulling...";

    runGitAction(async () => {
        const result = await gitInvoke(command);
        showGitNote(result.split("\n").pop() || doneText);
    }, startNote);
}


function openGitView() {
    showSidebarView("git");
    refreshGit();
}

function initSourceControl() {
    document.getElementById("git-activity-btn")?.addEventListener("click", openGitView);
    document.getElementById("git-refresh-btn")?.addEventListener("click", refreshGit);
    document.getElementById("git-commit-btn")?.addEventListener("click", commitGit);
    document.getElementById("git-push-btn")?.addEventListener("click", () => pushOrPullGit("git_push", "Pushed"));
    document.getElementById("git-pull-btn")?.addEventListener("click", () => pushOrPullGit("git_pull", "Pulled"));

    document.getElementById("git-stage-all-btn")?.addEventListener("click", () => {
        runGitAction(() => gitInvoke("git_stage", { paths: [] }));
    });
    document.getElementById("git-unstage-all-btn")?.addEventListener("click", () => {
        runGitAction(() => gitInvoke("git_unstage", { paths: [] }));
    });

    document.getElementById("git-init-btn")?.addEventListener("click", () => {
        runGitAction(() => gitInvoke("git_init"));
    });

    document.getElementById("git-message")?.addEventListener("keydown", (e) => {
        if ((e.ctrlKey || e.metaKey) && e.key === "Enter") {
            e.preventDefault();
            commitGit();
        }
    });

    document.addEventListener("keydown", (e) => {
        if ((e.ctrlKey || e.metaKey) && e.shiftKey && e.key.toLowerCase() === "g") {
            e.preventDefault();
            window.setSidebarVisible?.(true);
            openGitView();
        }
    });

    
    window.addEventListener("focus", refreshGit);
}

window.refreshGit = refreshGit;

if (document.readyState === "loading") {
    document.addEventListener("DOMContentLoaded", initSourceControl);
} else {
    initSourceControl();
}