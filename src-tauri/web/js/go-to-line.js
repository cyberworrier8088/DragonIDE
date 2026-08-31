

let isGoToLineOpen = false;

function openGoToLine() {

    const modal = document.getElementById("go-to-line-modal");
    const input = document.getElementById("line-input");
    const totalLinesSpan = document.getElementById("total-lines");


    if (!modal) return;

    modal.classList.remove("hidden");
    isGoToLineOpen = true;

    const editor = document.getElementById("code-editor");

    if (editor && editor.value) {
        const lineCount = editor.value.split("\n").length;

        totalLinesSpan.textContent = `Total lines: ${lineCount}`;
    } else {
        totalLinesSpan.textContent = "Total lines: 0";
    }

    input.focus();
    input.ariaSelected();
}


function closeGoToline() {
    const modal = document.getElementById("go-to-line-modal");

    if (modal) {
        modal.classList.add("hidden");
        isGoToLineOpen = false;
    }
}

function goToLineNumber(lineNum) {
    const editor = document.getElementById("code-editor");

    if (!editor || !editor.value) {
        console.warn("No editor or content available");
        return;
    }

    const lines = editor.value.split("\n");
    const totalLines = lines.length;

    if (lineNum < 1 || lineNum > totalLines) {
        console.warn(`Line ${lineNum} is out of range (1-${totalLines})`);
        return;
    }

    let cursorPos = 0;

    for (let i = 0; i < lineNum - 1; i++) {
        cursorPos += lines[i].length + 1;
    }


    editor.setSelectionRange(cursorPos, cursorPos);

    scrollEditorToLine(lineNum);

    editor.focus();

    closeGoToLine();

    console.log(`Jumped to line ${lineNum}`);
}

function scrollEditorToLine(lineNum) {
    const editor = document.getElementById("code-editor");
    const highlight = document.getElementById("code-highlight");

    if (!editor) return;

    const lineHeight = parseFloat(window.getComputedStyle(editor).lineHeight);

    const scrollTop = (lineNum - 1) * lineHeight;

    editor.scrollTop = scrollTop;
    if (highlight) {
        highlight.scrollTop = scrollTop;
    }

    const editorHeight = editor.clientHeight;

    const scrollTopCentered = scrollTop - (editorHeight / 2) + (lineHeight / 2);

    editor.scrollTop = Math.max(0, scrollTopCentered);

    if (highlight) {
        highlight.scrollTop = Math.max(0, scrollTopCentered);
    }
}

document.addEventListener("DOMContentLoaded", () => {

    const closeBtn = document.getElementById("close-go-to-line");

    if (closeBtn) {
        closeBtn.addEventListener("click", closeGoToline);
    }

    const backdrop = document.querySelector(".modal-backdrop");
    if (backdrop) {

        backdrop.addEventListener("click", (e) => {
            if (isGoToLineOpen && e.target === backdrop) {
                closeGoToline();
            }
        });
    }

    const goBtn = document.getElementById("go-btn");
    if (goBtn) {
        goBtn.addEventListener("click", () => {
            const input = document.getElementById("line-input");
            const lineNum = parseInt(input.value, 10);

            if (isNaN(lineNum) || lineNum < 1) {
                console.warn("Please enter a valid line number");
                return;
            }

            goToLineNumber(lineNum);
        });
    }

    const lineInput = document.getElementById("line-input");

    if (lineInput) {
        lineInput.addEventListener("keydown", (e) => {
            if (e.key === "Enter") {
                e.preventDefault();
                const lineNum = parseInt(lineInput.value, 10);

                if (isNaN(lineNum) || lineNum < 1) {
                    console.warn("Please enter a valid line number");
                    return;
                }

                goToLineNumber(lineNum);
            } else if (e.key === "Escape") {
                closeGoToline();
            }
        });
    }

    document.addEventListener("keydown", (e) => {
        if ((e.ctrlKey || e.metaKey) && e.key === "g") {
            e.preventDefault();
            openGoToLine();
        }
    });
});