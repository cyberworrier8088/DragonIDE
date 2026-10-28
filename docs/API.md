# Code AI API

A small HTTP API for **code autocomplete** (fill in the code at the cursor) and **chat**, running on
your own server. Plain JSON over HTTPS: call it from any language, app, bot or editor.

```
Base URL   https://imu.hackclub.app
Docs       https://imu.hackclub.app/docs        (this page)
Editor     https://imu.hackclub.app/            (try autocomplete in the browser)
Admin      https://imu.hackclub.app/admin       (make API keys, set token limits, see usage, test)
```

(No port in the address: Nest's HTTPS proxy forwards to the server's port 8787, which is not open to the
internet by itself.)

| Endpoint | What it does |
|---|---|
| `POST /v1/autocomplete` | Code to insert at the cursor, from the code before and after it. **Start here.** |
| `POST /v1/chat/completions` | Chat: questions, explanations, code from a description. OpenAI-compatible. |
| `POST /v1/completions` | Raw text/code continuation, optional fill-in-the-middle. OpenAI-compatible. |
| `GET /v1/usage` | Requests, tokens and dollar cost used by your key. |
| `GET /v1/models` | The models this server runs. |
| `GET /health` | Is the server up? (no key needed) |

Models: **Qwen2.5-Coder-0.5B** (autocomplete, completions) and **Qwen2.5-Coder-0.5B-Instruct** (chat).
They are small and fast: very good at finishing lines, function bodies and common patterns; chat answers
are short and simple. They know many languages (Python, JavaScript/TypeScript, Rust, Go, Java, C/C++, C#,
PHP, Ruby, SQL, HTML/CSS, shell, ...).

---

## 1. Authentication

Every `/v1/...` request needs your API key in a header:

```
Authorization: Bearer YOUR_KEY
```

(`X-Api-Key: YOUR_KEY` works too.) A missing or wrong key gets `401` with code `invalid_api_key`, a
disabled key `403` (`key_disabled`). Keys are never accepted in the URL.

Keys look like `sk-` followed by 48 characters. The server owner makes them in the admin panel
(section 9) and can give each key its own limits: tokens per day, output tokens per request, requests per
minute. Your key's limits and what you used today: `GET /v1/usage`.

**Keep the key secret.** Put it in an environment variable or a config file, not in code you share or in
a public web page. If a browser app needs the API, call it from your own backend, or only give the page
to people you trust. If a key leaks, the server owner replaces it (section 9).

---

## 2. Quick start

### curl

```bash
curl https://imu.hackclub.app/v1/autocomplete \
  -H "Authorization: Bearer $KEY" \
  -d '{"prefix": "def is_even(n):\n    return ", "suffix": "\n", "language": "python"}'
```

```json
{
  "completion": "n % 2 == 0",
  "model": "Qwen2.5-Coder-0.5B-Q4_0",
  "usage": {"prompt_tokens": 18, "completion_tokens": 7, "total_tokens": 25,
            "prompt_tokens_details": {"cached_tokens": 0}, "cost_usd": 1.24e-6},
  "timing_ms": {"queue": 0.1, "prompt": 92.0, "generate": 110.5, "total": 203.0}
}
```

### JavaScript (browser, Node 18+, Deno, Bun)

```js
const KEY = process.env.CODE_AI_KEY;           // never hard-code it in shared code

async function autocomplete(prefix, suffix = "", language = "") {
  const r = await fetch("https://imu.hackclub.app/v1/autocomplete", {
    method: "POST",
    headers: { Authorization: `Bearer ${KEY}`, "Content-Type": "application/json" },
    body: JSON.stringify({ prefix, suffix, language }),
  });
  const j = await r.json();
  if (!r.ok) throw new Error(`${j.error.code}: ${j.error.message}`);
  return j.cancelled ? null : j.completion;
}

console.log(await autocomplete("function add(a, b) {\n  return ", "\n}", "javascript"));  // "a + b;"
```

### Python (standard library only)

```python
import json, os, urllib.request

KEY = os.environ["CODE_AI_KEY"]

def call(path, body):
    req = urllib.request.Request("https://imu.hackclub.app" + path, json.dumps(body).encode(),
                                 {"Authorization": "Bearer " + KEY, "Content-Type": "application/json"})
    with urllib.request.urlopen(req, timeout=60) as r:
        return json.load(r)

print(call("/v1/autocomplete", {"prefix": "def area(r):\n    return ", "language": "python"})["completion"])
print(call("/v1/chat/completions", {"messages": [{"role": "user", "content": "What is a Python list?"}]})
      ["choices"][0]["message"]["content"])
```

### OpenAI libraries

Chat and completions follow the OpenAI format, so OpenAI SDKs work by changing the base URL:

```python
from openai import OpenAI            # pip install openai
ai = OpenAI(base_url="https://imu.hackclub.app/v1", api_key=KEY)
r = ai.chat.completions.create(model="Qwen2.5-Coder-0.5B-Instruct-Q4_0",
                               messages=[{"role": "user", "content": "Write a regex for an email address."}])
print(r.choices[0].message.content)
```

The `model` field is accepted but ignored: each endpoint has its own model.

---

## 3. POST /v1/autocomplete

Send the code before the cursor (`prefix`) and after it (`suffix`); get back the code to insert at the
cursor. This is what an editor's grey "ghost text" or inline suggestion shows.

### Request

| Field | Type | Default | Meaning |
|---|---|---|---|
| `prefix` | string | **required** | Code before the cursor. The model reads the last ~640 tokens (~2000 characters) of it. |
| `suffix` | string | `""` | Code after the cursor. The model reads the first ~48 tokens (~150 characters); a few hundred characters is plenty. |
| `language` | string | none | `python`, `javascript`, `typescript`, `rust`, `go`, `java`, `c`, `cpp`, `csharp`, `php`, `ruby`, `sql`, `html`, `css`, `bash`, ... or any file extension (`kt`, `dart`). |
| `filename` | string | none | Instead of `language`: e.g. `"src/app.tsx"`. The extension tells the model the language. |
| `max_lines` | integer 1-40 | `4` | Most lines in the suggestion. |
| `max_tokens` | integer 1-256 | `64` | Most tokens generated (about 3-4 characters each). |
| `single_line` | boolean | automatic | `true`: only finish the current line. Automatic: single line when there is text after the cursor on the same line, otherwise up to `max_lines`. |
| `stream` | boolean | `false` | Send the suggestion piece by piece while it forms (below). |

Without `language` or `filename` the model guesses the language from the code, which works most of the time.

### Response

```json
{
  "completion": "total / len(numbers)",
  "model": "Qwen2.5-Coder-0.5B-Q4_0",
  "usage": {
    "prompt_tokens": 47,
    "completion_tokens": 6,
    "total_tokens": 53,
    "prompt_tokens_details": {"cached_tokens": 41},
    "cost_usd": 7.8e-7
  },
  "timing_ms": {"queue": 0.2, "prompt": 48.1, "generate": 96.3, "total": 145.0}
}
```

* `completion` is the text to insert exactly at the cursor (it may start with a space or a newline and
  may contain newlines). An empty string means "no good suggestion here".
* `timing_ms.queue` is time spent waiting for other requests; `prompt` reading your code; `generate` writing.
* `cost_usd` is a JSON number; `7.8e-7` means $0.00000078.

**Newest request wins.** When a new `/v1/autocomplete` request arrives with the same key while an older
one is still running, the older one stops and answers `{"cancelled": true}` (with its `usage`, because some
work was done). This is exactly what you want while someone types: only the latest keystroke matters.
Use different keys for different users or apps so they do not cancel each other.

### Streaming

With `"stream": true` the answer is [server-sent events](https://developer.mozilla.org/docs/Web/API/Server-sent_events):
lines of `data: <json>` separated by a blank line. Each update holds the **whole suggestion so far**
(replace what you show, do not append). The last JSON has `"done": true` plus `usage` and `timing_ms`, then
`data: [DONE]` ends the stream.

```
data: {"completion":"total"}

data: {"completion":"total / len"}

data: {"completion":"total / len(numbers)"}

data: {"completion":"total / len(numbers)","done":true,"model":"Qwen2.5-Coder-0.5B-Q4_0","usage":{...},"timing_ms":{...}}

data: [DONE]
```

Closing the connection stops the generation (and its cost).

### Building an editor integration

The pattern every good autocomplete uses:

1. **Wait for a pause**: send a request 200-300 ms after the last keystroke, not on every key.
2. **Abort the old request** when a new keystroke comes (`AbortController` in JavaScript). The server also
   cancels it (newest wins), so you never pay for suggestions nobody will see.
3. **Skip** when the cursor is in the middle of a word or the file is empty.
4. **If the user types what the suggestion says, keep showing the rest** instead of asking again.
5. Accept with Tab: insert `completion` at the cursor.

```js
let timer, ctrl;
function onEdit(code, cursor, language, show) {          // call on every change; show(text) displays it
  clearTimeout(timer); ctrl?.abort();
  if (!code.trim() || /\w/.test(code[cursor] ?? "")) return show("");
  timer = setTimeout(async () => {
    ctrl = new AbortController();
    try {
      const r = await fetch("https://imu.hackclub.app/v1/autocomplete", {
        method: "POST", signal: ctrl.signal,
        headers: { Authorization: `Bearer ${KEY}`, "Content-Type": "application/json" },
        body: JSON.stringify({ prefix: code.slice(Math.max(0, cursor - 2000), cursor),
                               suffix: code.slice(cursor, cursor + 500), language }),
      });
      const j = await r.json();
      show(r.ok && !j.cancelled ? j.completion : "");
    } catch (e) { if (e.name !== "AbortError") show(""); }
  }, 250);
}
```

---

## 4. POST /v1/chat/completions

OpenAI-compatible chat with the Instruct model.

| Field | Type | Default | Meaning |
|---|---|---|---|
| `messages` | list | **required** | `[{"role": "system" \| "user" \| "assistant", "content": "..."}]`. `content` may also be OpenAI's list of `{"type": "text", "text": "..."}` parts. Without a system message, "You are a helpful assistant." is used. |
| `max_tokens` | integer 1-2048 | `256` | Most tokens in the answer (`max_completion_tokens` works too). |
| `temperature` | number 0-2 | `0.7` | Randomness. `0` = always the same, most likely answer. |
| `top_p` | number 0-1 | `0.8` | Nucleus sampling. |
| `stop` | string or list (max 8) | none | Stop when this text appears (not included in the answer). |
| `seed` | integer | random | Same seed + same request = same answer. |
| `stream` | boolean | `false` | Stream the answer in OpenAI's chunk format. |

```bash
curl https://imu.hackclub.app/v1/chat/completions -H "Authorization: Bearer $KEY" -d '{
  "messages": [
    {"role": "system", "content": "You are a senior Python reviewer. Be brief."},
    {"role": "user", "content": "Is `for i in range(len(xs))` good style?"}
  ],
  "max_tokens": 120
}'
```

```json
{
  "id": "chatcmpl-18e0332f941800cb",
  "object": "chat.completion",
  "created": 1792488931,
  "model": "Qwen2.5-Coder-0.5B-Instruct-Q4_0",
  "choices": [{"index": 0, "message": {"role": "assistant", "content": "Usually not: iterate directly ..."}, "finish_reason": "stop"}],
  "usage": {"prompt_tokens": 44, "completion_tokens": 58, "total_tokens": 102,
            "prompt_tokens_details": {"cached_tokens": 0}, "cost_usd": 7.1e-6}
}
```

`finish_reason` is `"stop"` (the answer ended, or a `stop` text appeared) or `"length"` (hit `max_tokens`).

**Streaming** (`"stream": true`): `data:` events with `{"choices": [{"delta": {"content": "..."}}]}` pieces
(append them), then a final chunk with `finish_reason` and `usage`, then `data: [DONE]`. Same as OpenAI.

The server keeps no conversation history: send the whole conversation every time (the server reuses the
work for the part it has already seen, so repeated history is fast and costs nothing extra while it stays cached).

---

## 5. POST /v1/completions

OpenAI-compatible text completion with the code model: continues `prompt`. With `suffix` it fills in the
middle like `/v1/autocomplete`, but without the editor-friendly trimming (use `stop`, e.g. `["\n\n"]`).

| Field | Type | Default | Meaning |
|---|---|---|---|
| `prompt` | string | **required** | Text to continue. |
| `suffix` | string | none | Text after the gap (fill in the middle). |
| `language` / `filename` | string | none | Language hint, as in `/v1/autocomplete` (used with `suffix`). |
| `max_tokens` | integer 1-2048 | `256` | |
| `temperature`, `top_p`, `stop`, `seed`, `stream` | | `0`, `1`, none, random, `false` | As in chat. |

```bash
curl https://imu.hackclub.app/v1/completions -H "Authorization: Bearer $KEY" \
  -d '{"prompt": "# Python: read a JSON file\nimport json\n\n", "max_tokens": 60, "stop": ["\n\n\n"]}'
```

Response: `{"choices": [{"index": 0, "text": "...", "finish_reason": "stop"}], "usage": {...}, ...}`.

---

## 6. GET /v1/usage

What your key has used since the counters started (they survive server restarts).

```json
{
  "key": "my-app",
  "requests": 1284,
  "prompt_tokens": 210443,
  "cached_tokens": 151870,
  "completion_tokens": 18920,
  "cost_usd": 0.0036,
  "today_tokens": 4120,
  "limits": {"daily_tokens": 50000, "max_output": 128, "rpm": 300},
  "since_unix": 1792488000,
  "prices_usd_per_1m_tokens": {"input": 0.03, "cached_input": 0.0, "output": 0.10}
}
```

* `today_tokens`: tokens counted against today's limit (since 00:00 UTC): prompt tokens the server had to
  read plus output tokens, the same tokens that cost money. Cached prompt tokens are free and do not count.
* `limits`: `daily_tokens` (tokens per day), `max_output` (most output tokens per request; a larger
  `max_tokens` is lowered to it), `rpm` (requests per minute). `0` means no limit.

Each key only sees its own usage. Every generation reply also carries its own `usage.cost_usd`.

---

## 7. Cost

The server is free to use (it runs on Hack Club Nest), but each reply shows what the work **would cost**
on a rented server, so you can see how expensive your app is and compare it with paid APIs.

| | Price per 1 million tokens |
|---|---|
| Input: prompt tokens the server had to read | **$0.03** |
| Cached input: prompt tokens it had already read (same start as your last request) | **$0.00** |
| Output: generated tokens | **$0.10** |

How these were chosen: a 2-CPU cloud server costs about $5 a month ($0.007 per hour). On this server the
model reads about 200 tokens per second and writes about 60 per second, so at full load that is about
$0.01 per million input and $0.03 per million output tokens. A server is not busy all the time, so the
prices are about 3 times that. For comparison, small paid models cost $0.10-0.60 per million tokens.

`cost_usd = (prompt_tokens - cached_tokens) × 0.03 / 1,000,000 + completion_tokens × 0.10 / 1,000,000`

| Typical request | Tokens | Cost |
|---|---|---|
| One autocomplete while typing | ~50 new prompt + 10 output | ~$0.0000025 |
| 1,000 autocompletes (a long coding session) | | ~$0.0025 |
| A chat answer | ~40 prompt + 150 output | ~$0.000016 |

A token is about 3-4 characters of code. A token count includes the whole prompt the model reads: for
autocomplete, your `prefix` and `suffix` plus a few tokens of framing.

---

## 8. Errors and limits

Errors are JSON with an HTTP status:

```json
{"error": {"code": "invalid_api_key", "message": "missing or wrong API key: send the header Authorization: Bearer <key>"}}
```

| Status | `code` | Meaning | What to do |
|---|---|---|---|
| 400 | `invalid_request` | Missing field, bad JSON, empty prompt. | Fix the request (the message says what). |
| 400 | `context_length_exceeded` | Prompt + `max_tokens` over 2048 tokens. | Send less text, or lower `max_tokens`. |
| 401 | `invalid_api_key` | No key or wrong key. | Send `Authorization: Bearer <key>`. |
| 403 | `key_disabled` | The owner disabled this key. | Ask the server owner. |
| 404 | `not_found` | No such endpoint. | Check the path. |
| 405 | `method_not_allowed` | e.g. GET on a POST endpoint. | |
| 411 | `length_required` | Body sent without `Content-Length` (chunked). | Send a normal body; every HTTP library does by default. |
| 413 | `payload_too_large` | Body over 1 MB. | Send less code (2000 characters around the cursor is plenty). |
| 429 | `rate_limited` | More requests this minute than the key allows (300 unless the owner set another limit). | Wait (`Retry-After` header); debounce autocomplete. |
| 429 | `quota_exceeded` | The key used its tokens for today. | Wait until 00:00 UTC, or ask the owner for a higher limit. |
| 503 | `server_busy` | Too many requests waiting, or too many open connections. | Retry after a few seconds (`Retry-After`). |
| 503 | `model_unavailable` | The chat model is not installed on this server. | Tell the server owner. |

Limits:

* **One answer at a time.** The server has 2 CPU cores; requests wait in a queue (up to 16), first come first served.
  A long chat answer makes autocomplete wait, so keep `max_tokens` small where you can.
* **Context:** prompt + answer up to 2048 tokens per request.
* **Speed:** about 60 output tokens per second; a typical autocomplete takes 0.2-0.5 s.
* **Body:** 1 MB. **Rate:** 300 requests per minute per key, unless the owner set another limit.
* **Per-key limits** set by the owner: tokens per day (checked before each request, so the last request of
  the day may go a little over), most output tokens per request, requests per minute.
* **Timeouts:** the server waits up to 10 s for the request to arrive. Set your client timeout to 60 s or more for long chat answers.
* **No uptime guarantee:** Nest is a free service. The server restarts by itself after a crash or reboot;
  retry once on a network error or `503`.

---

## 9. Admin panel (for the owner)

Open **https://imu.hackclub.app/admin** and sign in with the admin key (`SMOLLM_ADMIN_KEY`, see below).

* **Keys and usage:** every key with today's tokens (and a bar against its daily limit), requests, tokens in /
  cached / out, and cost; totals for the whole server.
* **New API key:** a name plus optional limits (tokens per day, output tokens per request, requests per
  minute; 0 = none). The key is shown **once**: copy it then. The server keeps only a SHA-256 hash of it,
  so a lost key cannot be shown again; make a new one.
* **Change limits** of a key (Save), **Disable / Enable** it (usage history stays), **Delete** it (apps
  using it stop at once; its usage history is removed).
* **Test:** run autocomplete or chat with the admin key (no limits) or with any API key (to check its
  limits), and see the answer, tokens, cost, time, the raw JSON and the same request as a `curl` command.

The same works without the page, for scripts (`Authorization: Bearer <admin key>`):

| Request | Does |
|---|---|
| `GET /admin/api/keys` | All keys with their limits and usage, plus totals. |
| `POST /admin/api/keys` `{"name": "my-app", "daily_tokens": 50000, "max_output": 128, "rpm": 60}` | New key; the reply's `"key"` is the secret (shown once). |
| `POST /admin/api/keys/<name>` `{"daily_tokens": 0, "disabled": true}` | Change any of `daily_tokens`, `max_output`, `rpm`, `disabled`. |
| `DELETE /admin/api/keys/<name>` | Delete the key. |

The admin key also works as an API key without limits (its usage shows as "admin").

## 10. Server settings (for the owner)

Set in `/opt/smollm/smollm.env` (secrets) and the systemd unit, then `systemctl restart smollm`.

| Variable | Default | Meaning |
|---|---|---|
| `SMOLLM_ADMIN_KEY` | none | Admin key for `/admin` (16+ characters). Without it the admin panel is off. |
| `SMOLLM_KEYS_FILE` | none | The key store (hashes and limits). On Nest: `/var/lib/smollm/keys.json`. |
| `SMOLLM_KEYS` | none | Keys to start the store with: `name:secret,name2:secret2` (secrets 16+ characters), added on the store's first start only; after that the admin panel is in charge. Without any key or admin key the server only listens on 127.0.0.1. |
| `SMOLLM_HOST` / `SMOLLM_PORT` | `127.0.0.1` / `8787` | Where it listens. |
| `SMOLLM_RPM` | `300` | Default requests per minute per key (`0` = no limit); a key's own limit wins. |
| `SMOLLM_CTX` | `2048` | Most tokens per request (prompt + answer). |
| `SMOLLM_PRICE_INPUT` / `SMOLLM_PRICE_OUTPUT` | `0.03` / `0.10` | Dollars per million tokens shown as cost. |
| `SMOLLM_USAGE_FILE` | none | Where usage counters are saved (kept across restarts). |
| `SMOLLM_CHAT_MODEL` | Instruct model | GGUF file for chat; empty turns chat off and saves ~390 MB RAM. |
| `SMOLLM_THREADS` | `2` | CPU threads. |
| `SMOLLM_SUFFIX` | `48` | Tokens of `suffix` the model reads (more = slower autocomplete). |

Make, limit and revoke API keys in the admin panel. To change the admin key, edit `SMOLLM_ADMIN_KEY` in
`/opt/smollm/smollm.env` (`head -c 24 /dev/urandom | od -An -tx1 | tr -d ' \n'` prints a random one) and
restart. Logs (one line per request and per admin change, never a secret): `journalctl -u smollm -f`.
