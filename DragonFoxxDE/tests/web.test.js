// Web UI checks, run with: node --test DragonFoxxDE/tests/web.test.js
const test = require("node:test");
const assert = require("node:assert");
const fs = require("node:fs");
const path = require("node:path");

const web = path.join(__dirname, "..", "web");

test("every file index.html loads exists", () => {
    const html = fs.readFileSync(path.join(web, "index.html"), "utf8");
    for (const [, file] of html.matchAll(/(?:src|href)="([^"#:]+)"/g)) {
        assert.ok(fs.existsSync(path.join(web, file)), `missing ${file}`);
    }
});

test("every language has the same keys as English", () => {
    const keys = file => Object.keys(JSON.parse(fs.readFileSync(path.join(web, "locales", file), "utf8"))).sort();
    for (const file of fs.readdirSync(path.join(web, "locales"))) {
        assert.deepStrictEqual(keys(file), keys("en.json"), file);
    }
});

// ai-complete.js is a plain browser script, so run it with small stand-ins for the browser
const store = {};
global.localStorage = { getItem: k => store[k] ?? null, setItem: (k, v) => { store[k] = String(v); } };
global.document = { addEventListener() {}, getElementById: () => null };
const ai = new Function(fs.readFileSync(path.join(web, "js", "ai-complete.js"), "utf8") +
    "; return { AI_PROVIDERS, aiProvider, aiStoreName, aiEnabled, callAi, cleanCompletion };")();

test("AI settings: old key name, unknown provider, local server without key", () => {
    assert.equal(ai.aiStoreName("key"), "ai-key");
    store["ai-provider"] = "bogus";
    assert.equal(ai.aiProvider(), "dragon");
    store["ai-provider"] = "custom";
    assert.equal(ai.aiEnabled(), true);
    store["ai-provider"] = "groq";
    assert.equal(ai.aiStoreName("key"), "ai-key-groq");
});

test("AI replies keep only the new part of the line", () => {
    const clean = ai.cleanCompletion;
    assert.equal(clean("```python\n    return a + b\n```", "    return a"), " + b");
    assert.equal(clean("return a + b", "    return a"), " + b");
    assert.equal(clean(" + b", "    return a"), " + b");
    assert.equal(clean("\n    return 1", "    "), "return 1");
    assert.equal(clean("return 1", "    "), "return 1");
    assert.equal(clean("\n    return 1", "def f():"), "");
});

test("AI request and errors (fake Groq)", async () => {
    store["ai-provider"] = "groq";
    store["ai-key-groq"] = "test";
    let sent;
    global.fetch = async (url, options) => {
        sent = { url, ...options };
        return { ok: true, json: async () => ({ choices: [{ message: { content: "x" } }] }) };
    };
    assert.equal(await ai.callAi("a", "b", "t.py"), "x");
    assert.equal(sent.url, ai.AI_PROVIDERS.groq.url);
    assert.equal(sent.headers.Authorization, "Bearer test");

    global.fetch = async () => ({ ok: false, status: 401, json: async () => ({ error: { message: "Invalid API Key" } }) });
    await assert.rejects(ai.callAi("a", "b", "t.py"), /Invalid API Key/);
    global.fetch = async () => ({ ok: false, status: 502, json: async () => { throw new SyntaxError("html"); } });
    await assert.rejects(ai.callAi("a", "b", "t.py"), /Error 502/);
    global.fetch = async () => { throw new TypeError("Failed to fetch"); };
    await assert.rejects(ai.callAi("a", "b", "t.py"), /Can't reach the AI server/);
});
