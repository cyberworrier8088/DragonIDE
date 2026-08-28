let currentTranslations = {};


async function loadLanguage(lang) {

    try {
        const response = await fetch(`./locales/${lang}.json`);

        if (!response.ok) {
            throw new Error(`Failed to load ${lang}`);
        }

        currentTranslations = await response.json();

    } catch (error) {

        console.error("i18n load Error: ", error);

        if (lang !== "en") {
            await loadLanguage("en");
        }
    }

}


function t(key, params = {}) {
    let text = currentTranslations[key] || key;

    Object.keys(params).forEach(param => {
        text = text.replace(`{${param}}`, params[param]);
    });

    return text;
}

function updateDOMTranslations() {
    document.querySelectorAll("[data-i18n]").forEach(element => {
        const key = element.getAttribute("data-i18n");
        element.textContent = t(key);
    });
}

async function setLanguage(lang) {

    await loadLanguage(lang);
    updateDOMTranslations();
    document.documentElement.lang = lang;

    if (typeof settings != "undefined") {
        settings.language = lang;
        saveSettings(settings);
    }

    window.dispatchEvent(new CustomEvent("Language-changed", { detail: lang }));
}
