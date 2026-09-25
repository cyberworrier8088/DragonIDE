

let findReplaceState = {
    isOpen: false,
    currentMatchIndex: 0,
    matches: [],
    lastFindText: '',
    lastReplaceText: '',
    caseSensitive: false,
    wholeWord: false,
    regex: false
};

function openFindReplace() {
    const modal = document.getElementById("find-replace-modal");
    const input = document.getElementById("find-input");

    if (!modal) return;

    modal.classList.remove("hidden");
    findReplaceState.isOpen = true;
    input.focus();
    input.select();
}

function closeFindReplace() {
    const modal = document.getElementById("find-replace-modal");

    if (modal) {
        modal.classList.add("hidden");
        findReplaceState.isOpen = false;
    }

    clearHighlights();
}


function getFindOptions() {
    const caseSensitive = document.getElementById("case-sensitive-check");
    const wholeWord = document.getElementById("whole-word-check");
    const regex = document.getElementById("regex-check");

    return {
        caseSensitive: caseSensitive ? caseSensitive.checked : false,
        wholeWord: wholeWord ? wholeWord.checked : false,
        regex: regex ? regex.checked : false
    };
}


function findMatches(text, searchText, options) {

    if (!searchText || !text) return [];

    let matches = [];
    let regex;

    try {
        if (options && options.regex) {
            const flags = options.caseSensitive ? 'g' : 'gi';
            regex = new RegExp(searchText, flags);
        } else {
            const escapedSearch = searchText.replace(/[.*+?^${}()|[\]\\]/g, '\\$&');

            const pattern = (options && options.wholeWord) ? `\\b${escapedSearch}\\b` : escapedSearch;

            const flags = (options && options.caseSensitive) ? 'g' : 'gi';
            regex = new RegExp(pattern, flags);
        }

        let match;

        while ((match = regex.exec(text)) !== null) {
            matches.push({
                start: match.index,
                end: match.index + match[0].length,
                text: match[0]
            });
            if (regex.lastIndex === match.index) {
                regex.lastIndex++;
            }
        }
    } catch (e) {
        console.error("Regex error:", e);
        return [];
    }

    return matches;
}

function highlightMatches() {
    const editor = document.getElementById("code-editor");
    const highlight = document.getElementById("code-highlight");
    const findInput = document.getElementById("find-input");
    if (!editor || !findInput) return;

    const searchText = findInput.value;

    if (!searchText || !editor.value) {
        findReplaceState.matches = [];
        findReplaceState.currentMatchIndex = 0;
        updateMatchCounter(0, 0);
        if (typeof syncHighlight === 'function') {
            syncHighlight();
        }
        return;
    }

    const options = getFindOptions();
    findReplaceState.matches = findMatches(editor.value, searchText, options);
    findReplaceState.currentMatchIndex = 0;

    updateMatchCounter(
        findReplaceState.matches.length > 0 ? 1 : 0,
        findReplaceState.matches.length
    );

    if (highlight && findReplaceState.matches.length > 0) {
        highlightInPreTag(highlight, findReplaceState.matches, 0);
    } else if (typeof syncHighlight === 'function') {
        syncHighlight();
    }
}

function highlightInPreTag(preElement, matches, currentIndex) {

    const codeElement = preElement.querySelector('code');

    if (!codeElement) return;

    const text = codeElement.textContent;
    const fragments = [];
    let lastIndex = 0;

    matches.forEach((match, index) => {

        if (match.start > lastIndex) {
            const span = document.createElement('span');
            span.textContent = text.substring(lastIndex, match.start);
            fragments.push(span);
        }

        const highlightSpan = document.createElement('span');
        highlightSpan.textContent = text.substring(match.start, match.end);
        highlightSpan.className = index === currentIndex ? 'find-highlight current' : 'find-highlight';
        fragments.push(highlightSpan);

        lastIndex = match.end;
    });


    if (lastIndex < text.length) {
        const span = document.createElement('span');
        span.textContent = text.substring(lastIndex);
        fragments.push(span);
    }

    codeElement.innerHTML = '';
    fragments.forEach(fragment => codeElement.appendChild(fragment));

}


function clearHighlights() {
    findReplaceState.matches = [];
    findReplaceState.currentMatchIndex = 0;
    updateMatchCounter(0, 0);

    const highlight = document.getElementById('code-highlight');
    if (highlight) {
        const codeElement = highlight.querySelector('code');
        if (codeElement && typeof syncHighlight === 'function') {
            syncHighlight();
        }
    }
}

function clearHighlight() {
    clearHighlights();
}

function updateMatchCounter(current, total) {
    const counter = document.getElementById("match-counter");

    if (counter) {
        counter.textContent = total > 0 ? `${current}/${total}` : '0/0';
    }
}

function findNext() {

    if (findReplaceState.matches.length === 0) return;

    findReplaceState.currentMatchIndex = (findReplaceState.currentMatchIndex + 1) % findReplaceState.matches.length;

    navigateToMatch(findReplaceState.currentMatchIndex);
}

function findPrev() {
    if (findReplaceState.matches.length === 0) return;

    findReplaceState.currentMatchIndex--;
    if (findReplaceState.currentMatchIndex < 0) {
        findReplaceState.currentMatchIndex = findReplaceState.matches.length - 1;


    }


    navigateToMatch(findReplaceState.currentMatchIndex);
}

function navigateToMatch(index) {

    if (index < 0 || index >= findReplaceState.matches.length) return;


    const editor = document.getElementById("code-editor")
    const match = findReplaceState.matches[index];

    editor.setSelectionRange(match.start, match.end);

    scrollToMatch(match);

    const highlight = document.getElementById("code-highlight");
    highlightInPreTag(highlight, findReplaceState.matches, index);

    updateMatchCounter(index + 1, findReplaceState.matches.length);

    editor.focus();
}


function scrollToMatch(match) {
    const editor = document.getElementById("code-editor");
    const highlight = document.getElementById("code-highlight");

    const lineNum = editor.value.substring(0, match.start).split('\n').length;

    const lineHeight = parseFloat(window.getComputedStyle(editor).lineHeight) || 20;
    const scrollTop = (lineNum - 1) * lineHeight;
    const editorHeight = editor.clientHeight;
    const scrollTopCentered = scrollTop - (editorHeight / 2) + (lineHeight / 2);

    editor.scrollTop = Math.max(0, scrollTopCentered);

    if (highlight) {
        highlight.scrollTop = Math.max(0, scrollTopCentered);
    }


}


function replaceOne() {
    if (!findReplaceState.matches || findReplaceState.matches.length === 0 || findReplaceState.currentMatchIndex >= findReplaceState.matches.length) {
        return;
    }

    const editor = document.getElementById("code-editor");
    if (!editor) return;

    const match = findReplaceState.matches[findReplaceState.currentMatchIndex];
    const replaceInput = document.getElementById("replace-input");
    const replaceText = replaceInput ? replaceInput.value : "";

    const before = editor.value.substring(0, match.start);
    const after = editor.value.substring(match.end);
    editor.value = before + replaceText + after;

    editor.dispatchEvent(new Event('input', { bubbles: true }));

    highlightMatches();

    if (findReplaceState.matches.length > 0) {
        findNext();
    }
}

function replaceAll() {
    const editor = document.getElementById("code-editor");
    const findInput = document.getElementById("find-input");
    const replaceInput = document.getElementById("replace-input");
    if (!editor || !findInput) return;

    const replaceText = replaceInput ? replaceInput.value : "";
    const searchText = findInput.value;
    const options = getFindOptions();

    if (!searchText) return;

    let newText = editor.value;

    try {
        if (options.regex) {

            const flags = options.caseSensitive ? 'g' : 'gi';
            const regex = new RegExp(searchText, flags);
            newText = newText.replace(regex, replaceText);
        } else {
            const escapedSearch = searchText.replace(/[.*+?^${}()|[\]\\]/g, '\\$&');
            const pattern = options.wholeWord ? `\\b${escapedSearch}\\b` : escapedSearch;
            const flags = options.caseSensitive ? 'g' : 'gi';
            const regex = new RegExp(pattern, flags);
            newText = newText.replace(regex, replaceText);
        }
    } catch (e) {
        console.error("Regex Error:", e);
        return;
    }

    editor.value = newText;

    editor.dispatchEvent(new Event('input', { bubbles: true }));

    highlightMatches();
}


document.addEventListener("DOMContentLoaded", () => {

    const closeBtn = document.getElementById("close-find-replace");
    if (closeBtn) {
        closeBtn.addEventListener("click", closeFindReplace);
    }

    document.addEventListener("click", (e) => {
        const modal = document.getElementById("find-replace-modal");
        if (findReplaceState.isOpen && modal && (e.target.classList.contains("modal-backdrop") && e.target.closest("#find-replace-modal"))) {
            closeFindReplace();
        }
    });

    const findInput = document.getElementById("find-input");
    if (findInput) {
        findInput.addEventListener("input", () => {
            highlightMatches();
        });

        findInput.addEventListener("keydown", (e) => {
            if (e.key === "Enter") {
                e.preventDefault();
                findNext();
            } else if (e.key === "Escape") {
                closeFindReplace();
            }
        });
    }

    const replaceInput = document.getElementById("replace-input");

    if (replaceInput) {

        replaceInput.addEventListener("keydown", (e) => {
            if (e.key === "Enter") {
                e.preventDefault();
                replaceOne();
            } else if (e.key === "Escape") {
                closeFindReplace();
            }
        });
    }

    const prevBtn = document.getElementById("find-prev-btn");
    const nextBtn = document.getElementById("find-next-btn");

    if (prevBtn) prevBtn.addEventListener("click", findPrev);
    if (nextBtn) nextBtn.addEventListener("click", findNext);

    const replaceBtn = document.getElementById("replace-btn");
    const replaceAllBtn = document.getElementById("replace-all-btn");

    if (replaceBtn) replaceBtn.addEventListener("click", replaceOne);
    if (replaceAllBtn) replaceAllBtn.addEventListener("click", replaceAll);

    const caseSensitiveCheck = document.getElementById("case-sensitive-check");
    const wholeWordCheck = document.getElementById("whole-word-check");
    const regexCheck = document.getElementById("regex-check");

    [caseSensitiveCheck, wholeWordCheck, regexCheck].forEach(checkbox => {
        if (checkbox) {
            checkbox.addEventListener("change", highlightMatches);
        }
    });

    document.addEventListener("keydown", (e) => {

        if ((e.ctrlKey || e.metaKey) && e.key === "f") {
            e.preventDefault();
            openFindReplace();
        }

        if ((e.ctrlKey || e.metaKey) && e.key === "h") {
            e.preventDefault();
            openFindReplace();
        }
    });
});