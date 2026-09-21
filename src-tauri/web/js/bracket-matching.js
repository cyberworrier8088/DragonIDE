let bracketState = {
    isEnabled: true,
    autoCloseEnabled: true,
    matchedBrackets: [],
    currentBracketIndex: -1,
    bracketPairs: {
        '(': ')',
        '[': ']',
        '{': '}',
        '"': '"',
        "'": "'",
        '`': '`'
    },
    openingBrackets: ['(', '[', '{'],
    closingBrackets: [')', ']', '}'],
    autoCloseBrackets: ['(', '[', '{', '"', "'", '`']
};

function loadBracketSettings() {
    const saved = localStorage.getItem("bracketSettings");
    if (saved) {
        try {
            const settings = JSON.parse(saved);
            bracketState.autoCloseEnabled = settings.autoCloseEnabled !== false;
        } catch (e) {
            console.error("Failed to parse bracketSettings", e);
        }
    }
}

function saveBracketSettings() {
    const settings = {
        autoCloseEnabled: bracketState.autoCloseEnabled
    };
    localStorage.setItem("bracketSettings", JSON.stringify(settings));
}

function findMatchingBracket(text, position) {
    const char = text[position];
    if (!char) return -1;

    if (bracketState.openingBrackets.includes(char)) {
        const closingChar = bracketState.bracketPairs[char];
        if (!closingChar) return -1;
        let depth = 1;
        for (let i = position + 1; i < text.length; i++) {
            if (text[i] === char) depth++;
            else if (text[i] === closingChar) {
                depth--;
                if (depth === 0) return i;
            }
        }
    } else if (bracketState.closingBrackets.includes(char)) {
        const openingChar = Object.keys(bracketState.bracketPairs).find(
            key => bracketState.bracketPairs[key] === char
        );
        if (!openingChar) return -1;
        let depth = 1;
        for (let i = position - 1; i >= 0; i--) {
            if (text[i] === char) depth++;
            else if (text[i] === openingChar) {
                depth--;
                if (depth === 0) return i;
            }
        }
    }

    return -1;
}

function highlightMatchingBrackets(editor) {
    if (!editor) return;

    const text = editor.value || "";
    const cursorPos = editor.selectionStart;
    const charBefore = cursorPos > 0 ? text[cursorPos - 1] : null;
    const charAt = text[cursorPos];

    bracketState.matchedBrackets = [];

    let bracketPos = -1;
    if (charBefore && (bracketState.openingBrackets.includes(charBefore) || bracketState.closingBrackets.includes(charBefore))) {
        bracketPos = cursorPos - 1;
    } else if (charAt && (bracketState.openingBrackets.includes(charAt) || bracketState.closingBrackets.includes(charAt))) {
        bracketPos = cursorPos;
    }

    if (bracketPos === -1) return;

    const matchingPos = findMatchingBracket(text, bracketPos);
    if (matchingPos !== -1) {
        bracketState.matchedBrackets = [
            { pos: bracketPos, char: text[bracketPos] },
            { pos: matchingPos, char: text[matchingPos] }
        ];

        highlightBracketPair(bracketPos, matchingPos);
    }
}

function highlightBracketPair(openPos, closePos) {
    const highlightInner = document.getElementById("code-highlight-inner");
    if (!highlightInner) return;

    const textNodes = [];
    const walker = document.createTreeWalker(highlightInner, NodeFilter.SHOW_TEXT, null, false);
    let node;
    let offset = 0;
    while ((node = walker.nextNode())) {
        const len = node.textContent.length;
        textNodes.push({ node, start: offset, end: offset + len });
        offset += len;
    }

    const targets = [
        { pos: openPos, className: "bracket-highlight current" },
        { pos: closePos, className: "bracket-highlight" }
    ].sort((a, b) => b.pos - a.pos);

    for (const target of targets) {
        const entry = textNodes.find(item => target.pos >= item.start && target.pos < item.end);
        if (!entry) continue;

        const offsetInNode = target.pos - entry.start;
        const textNode = entry.node;
        if (offsetInNode >= textNode.textContent.length) continue;

        const charSpan = document.createElement("span");
        charSpan.className = target.className;
        charSpan.textContent = textNode.textContent.charAt(offsetInNode);

        const afterNode = textNode.splitText(offsetInNode);
        afterNode.textContent = afterNode.textContent.substring(1);
        afterNode.parentNode.insertBefore(charSpan, afterNode);
    }
}

function jumpToMatchingBracket(ed) {
    const editor = ed || document.getElementById("code-editor");
    if (!editor) return;

    const text = editor.value || "";
    const cursorPos = editor.selectionStart;
    if (cursorPos === 0) return;

    const charAtCursor = text[cursorPos - 1];
    const charAtPos = text[cursorPos];

    let bracketPos;
    if (bracketState.bracketPairs[charAtCursor] || bracketState.closingBrackets.includes(charAtCursor)) {
        bracketPos = cursorPos - 1;
    } else if (bracketState.bracketPairs[charAtPos] || bracketState.closingBrackets.includes(charAtPos)) {
        bracketPos = cursorPos;
    } else {
        console.log("Not on a bracket");
        return;
    }

    const matchingPos = findMatchingBracket(text, bracketPos);
    if (matchingPos !== -1) {
        editor.setSelectionRange(matchingPos, matchingPos + 1);
        editor.focus();
        scrollToBracket(matchingPos);
        highlightBracketPair(bracketPos, matchingPos);
    }
}

function scrollToBracket(pos) {
    const editor = document.getElementById("code-editor");
    const highlight = document.getElementById("code-highlight");
    if (!editor) return;

    const lineNum = editor.value.substring(0, pos).split('\n').length;
    const lineHeight = parseFloat(window.getComputedStyle(editor).lineHeight) || 20;
    const scrollTop = (lineNum - 1) * lineHeight;
    const editorHeight = editor.clientHeight;
    const scrollTopCentered = scrollTop - (editorHeight / 2) + (lineHeight / 2);

    editor.scrollTop = Math.max(0, scrollTopCentered);
    if (highlight) {
        highlight.scrollTop = Math.max(0, scrollTopCentered);
    }
}

function handleBracketInput(event) {
    if (!bracketState.autoCloseEnabled) return;

    const editor = document.getElementById("code-editor");
    if (!editor) return;

    const char = event.key;
    if (!bracketState.autoCloseBrackets.includes(char)) return;

    const text = editor.value;
    const cursorPos = editor.selectionStart;
    const selectedText = editor.value.substring(editor.selectionStart, editor.selectionEnd);

    if (bracketState.closingBrackets.includes(char)) {
        if (text[cursorPos] === char) {
            event.preventDefault();
            editor.setSelectionRange(cursorPos + 1, cursorPos + 1);
            return;
        }
    }

    if (selectedText.length > 0 && bracketState.openingBrackets.includes(char)) {
        event.preventDefault();
        const closingChar = bracketState.bracketPairs[char];
        const before = text.substring(0, editor.selectionStart);
        const after = text.substring(editor.selectionEnd);

        editor.value = before + char + selectedText + closingChar + after;
        editor.setSelectionRange(editor.selectionStart + 1, editor.selectionStart + 1 + selectedText.length);
        editor.dispatchEvent(new Event('input', { bubbles: true }));
        return;
    }

    if (bracketState.openingBrackets.includes(char)) {
        event.preventDefault();
        const closingChar = bracketState.bracketPairs[char];
        const before = text.substring(0, cursorPos);
        const after = text.substring(cursorPos);

        editor.value = before + char + closingChar + after;
        editor.setSelectionRange(cursorPos + 1, cursorPos + 1);
        editor.dispatchEvent(new Event('input', { bubbles: true }));
        return;
    }

    if ((char === '"' || char === "'" || char === '`') && (text[cursorPos] === ' ' || text[cursorPos] === '\n' || !text[cursorPos] || /[,;)}\]"]/.test(text[cursorPos]))) {
        event.preventDefault();
        const before = text.substring(0, cursorPos);
        const after = text.substring(cursorPos);

        editor.value = before + char + char + after;
        editor.setSelectionRange(cursorPos + 1, cursorPos + 1);
        editor.dispatchEvent(new Event('input', { bubbles: true }));
    }
}

document.addEventListener("DOMContentLoaded", () => {
    loadBracketSettings();

    const editor = document.getElementById("code-editor");
    if (editor) {
        editor.addEventListener("keypress", handleBracketInput);

        editor.addEventListener("click", () => {
            if (typeof syncHighlight === 'function') {
                syncHighlight();
            } else {
                highlightMatchingBrackets(editor);
            }
        });

        editor.addEventListener("keyup", (e) => {
            if (["ArrowLeft", "ArrowRight", "ArrowUp", "ArrowDown", "Home", "End", "PageUp", "PageDown"].includes(e.key)) {
                if (typeof syncHighlight === 'function') {
                    syncHighlight();
                } else {
                    highlightMatchingBrackets(editor);
                }
            }
        });

        editor.addEventListener("blur", () => {
            bracketState.matchedBrackets = [];
        });
    }

    document.addEventListener("keydown", (e) => {
        if ((e.ctrlKey || e.metaKey) && e.shiftKey && e.key === "\\") {
            e.preventDefault();
            jumpToMatchingBracket();
        }
    });

    const autoBracketsToggle = document.getElementById("auto-brackets-toggle");
    if (autoBracketsToggle) {
        autoBracketsToggle.checked = bracketState.autoCloseEnabled;
        autoBracketsToggle.addEventListener("change", () => {
            bracketState.autoCloseEnabled = autoBracketsToggle.checked;
            saveBracketSettings();
            console.log("Auto-Close brackets:", bracketState.autoCloseEnabled ? "enabled" : "disabled");
        });
    }
});

window.jumpToMatchingBracket = jumpToMatchingBracket;
window.highlightMatchingBrackets = highlightMatchingBrackets;
window.highlightMatchedBrackets = highlightMatchingBrackets;