// Universal Theme-Reactive SVG Icon System
// Automatically converts any SVG <img> or .theme-svg into an inline <svg> with currentColor,
// so any SVG icon seamlessly adapts to every theme (Dark, Light, Monokai, Nord, Glassmorphism).

const svgContentCache = new Map();

async function convertSvgImg(img) {
    if (img.dataset.svgProcessed === "true") return;
    const src = img.getAttribute("src") || img.getAttribute("data-src");
    if (!src || !src.includes(".svg")) return;

    img.dataset.svgProcessed = "true";

    try {
        let svgText = svgContentCache.get(src);
        if (!svgText) {
            const res = await fetch(src);
            if (!res.ok) return;
            svgText = await res.text();
            svgContentCache.set(src, svgText);
        }

        const parser = new DOMParser();
        const doc = parser.parseFromString(svgText, "image/svg+xml");
        const svg = doc.querySelector("svg");
        if (!svg) return;

        // Auto-adapt colors: convert hardcoded black/white strokes & fills to currentColor
        const normalizeColor = (val) => {
            if (!val || val === "none") return val;
            const lower = val.trim().toLowerCase();
            if (lower === "#fff" || lower === "#ffffff" || lower === "white" ||
                lower === "#000" || lower === "#000000" || lower === "black" ||
                lower.startsWith("#1") || lower.startsWith("#2") || lower.startsWith("#e") || lower.startsWith("#f")) {
                return "currentColor";
            }
            return val;
        };

        svg.querySelectorAll("*").forEach(el => {
            if (el.hasAttribute("stroke")) {
                const s = el.getAttribute("stroke");
                if (s !== "none") el.setAttribute("stroke", "currentColor");
            }
            if (el.hasAttribute("fill")) {
                const f = el.getAttribute("fill");
                if (f !== "none") el.setAttribute("fill", "currentColor");
            }
        });

        if (svg.hasAttribute("stroke") && svg.getAttribute("stroke") !== "none") {
            svg.setAttribute("stroke", "currentColor");
        }
        if (svg.hasAttribute("fill") && svg.getAttribute("fill") !== "none") {
            svg.setAttribute("fill", "currentColor");
        }

        // Copy classes, IDs, dimensions, and tooltips
        if (img.className) svg.setAttribute("class", img.className);
        if (img.id) svg.setAttribute("id", img.id);
        if (img.title) svg.setAttribute("title", img.title);
        if (img.alt && !svg.getAttribute("title")) svg.setAttribute("title", img.alt);

        const width = img.getAttribute("width") || (img.style.width ? parseInt(img.style.width, 10) : null) || 22;
        const height = img.getAttribute("height") || (img.style.height ? parseInt(img.style.height, 10) : null) || 22;
        svg.setAttribute("width", width);
        svg.setAttribute("height", height);

        img.replaceWith(svg);
    } catch (err) {
        console.warn("SVG auto-conversion failed for:", src, err);
    }
}

function convertSvgIcons(root = document) {
    const images = root.querySelectorAll('img[src*=".svg"], img[data-src*=".svg"], img.theme-svg');
    images.forEach(img => convertSvgImg(img));
}

// Watch DOM for any dynamically added SVG icons now and in the future
const svgObserver = new MutationObserver((mutations) => {
    for (const mutation of mutations) {
        for (const node of mutation.addedNodes) {
            if (node.nodeType === Node.ELEMENT_NODE) {
                if (node.matches && (node.matches('img[src*=".svg"]') || node.matches('img[data-src*=".svg"]'))) {
                    convertSvgImg(node);
                } else if (node.querySelectorAll) {
                    convertSvgIcons(node);
                }
            }
        }
    }
});

if (document.readyState === "loading") {
    document.addEventListener("DOMContentLoaded", () => {
        convertSvgIcons();
        if (document.body) {
            svgObserver.observe(document.body, { childList: true, subtree: true });
        }
    });
} else {
    convertSvgIcons();
    if (document.body) {
        svgObserver.observe(document.body, { childList: true, subtree: true });
    }
}

window.convertSvgIcons = convertSvgIcons;
