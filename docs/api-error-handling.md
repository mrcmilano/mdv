# API Error Handling in the UI

Scope: any frontend code that makes API calls — fetch, HTTP client wrappers,
data-fetching hooks, form submissions, mutations. Apply every rule below before
writing or modifying that code.

---

## 1. Classify the error first

Every non-2xx response belongs to exactly one category. Handle each differently.
Never apply a single generic handler across all of them.

| Status | Category | Required action |
|---|---|---|
| `400` | Validation / bad request | Inline field-level message — no retry |
| `401` | Unauthenticated | Redirect to login — no toast |
| `403` | Forbidden | Access-denied message — no retry |
| `404` | Not found | Empty / not-found state — no retry |
| `408` `429` `5xx` | Transient | Retry with backoff (§3) |
| Network failure | Offline / timeout | Retry with backoff + offline check (§7) |

**Rule:** never retry `4xx` errors except `408` and `429` — they will not resolve on their own.

---

## 2. Centralize error interception

Use one HTTP client instance with a single response interceptor.
Do not scatter status-code logic across components.
Every branch must `return` — missing a return causes fall-through and double-handling.

```ts
// httpClient.ts
client.interceptors.response.use(
  (res) => res,
  (error) => {
    const status = error.response?.status;

    if (!error.response)  return Promise.reject(new NetworkError());
    if (status === 401)   return redirectToLogin();              // return — stops fall-through
    if (status === 403)   return Promise.reject(notifyAndReject("You don't have access to this."));
    if (status >= 400 && status < 500)
      return Promise.reject(new ClientError(status, error.response.data));

    return Promise.reject(error); // 5xx → bubble up for retry
  }
);
```

Components handle only the application-level meaning of an error, not its HTTP mechanics.

---

## 3. Retry transient errors with exponential backoff

Retry only `408`, `429`, `5xx`, and network failures. Cap at **3 attempts**.
Add jitter to prevent thundering-herd. Respect `Retry-After` on `429` when present.

```ts
async function fetchWithRetry<T>(
  fn: () => Promise<T>,
  { maxRetries = 3, baseDelay = 1000, maxDelay = 30_000 } = {}
): Promise<T> {
  for (let attempt = 0; attempt <= maxRetries; attempt++) {
    try {
      return await fn();
    } catch (err) {
      if (!isRetryable(err) || attempt === maxRetries) throw err; // 408|429|5xx|network
      const delay = Math.min(2 ** attempt * baseDelay + Math.random() * 500, maxDelay);
      await sleep(delay); // 1 s → 2 s → 4 s (+ jitter)
    }
  }
  throw new Error("unreachable");
}
```

---

## 4. Show the right UI for each error state

**Transient / retrying**
- Non-blocking toast or inline status: `"Reconnecting… attempt 2 of 3"`.
- Keep existing content visible — do not replace the page with an error screen.

**Fatal (retries exhausted, or non-retryable)**
- Replace the failed *section*, not the whole page, with an error boundary fallback.
- Include a **"Try again"** button for recoverable errors.
- `401` → redirect silently (no toast).
- `403` / `404` → static message with a link to a safe next step.

**Validation (`400`)**
- Inline, field-level messages when the API returns field info.
- Never display raw server messages — map them to plain-language copy.

**Partial failure**
- If a non-critical section fails (sidebar, related content), show a local fallback.
- Do not degrade the main content area.

---

## 5. User-facing message rules

Never expose: status codes, stack traces, SQL errors, internal field names, API keys.
Always communicate: what happened, whether it is temporary, what the user can do next.

```
✗  "Error 500: Internal Server Error"
✓  "Something went wrong on our end — please try again shortly."

✗  "VALIDATION_FAILED: email must match /^[^@]+@[^@]+$/"
✓  "Please enter a valid email address."
```

Use two layers: a friendly UI message shown to the user, and full error detail sent
to the monitoring service (Sentry, Datadog, etc.).

---

## 6. Optimistic updates — roll back on failure

Apply state changes immediately; revert if the request fails.

```ts
const prev = getState();
setState(optimistic);

try {
  await api.save(payload);
} catch (err) {
  setState(prev);
  notifyUser("Couldn't save. Please try again.");
}
```

---

## 7. Offline detection

`navigator.onLine` returns `true` when connected to *any* network, not necessarily
the internet — VPNs and captive portals cause false positives.
Treat `false` as definitive (offline). Verify `true` with a `HEAD` ping before acting.

```ts
async function isReachable(): Promise<boolean> {
  if (!navigator.onLine) return false;
  try {
    const url = new URL(window.location.origin);
    url.searchParams.set("_", Date.now().toString()); // bust cache
    return (await fetch(url, { method: "HEAD", cache: "no-store" })).ok;
  } catch {
    return false;
  }
}
```

Listen for `online` / `offline` events but re-run `isReachable()` on `online` before
resuming the queue — the event fires on interface changes, not internet restoration.

When confirmed offline:
- Pause the request queue silently.
- Show a persistent banner: `"You're offline — changes will sync when reconnected."`.
- Resume only after `isReachable()` returns `true`.

---

## 8. Cancel stale requests with `AbortController`

Without cancellation, a slow earlier request can resolve after a faster later one,
overwriting fresh state with stale data. Tie every request to a controller.

**General pattern:**
```ts
const controller = new AbortController();
fetch(url, { signal: controller.signal })
  .catch((err) => {
    if (err.name === "AbortError") return; // not a real error — swallow silently
    handleError(err);
  });
controller.abort(); // call on supersession or cleanup
```

**React — cancel on unmount:**
```ts
useEffect(() => {
  const controller = new AbortController();
  fetchData(controller.signal)
    .then(setData)
    .catch((err) => { if (err.name !== "AbortError") setError(err); });
  return () => controller.abort(); // runs on unmount and before next effect
}, [query]);
```

`AbortError` is not a failure. Never log it or show it in the UI.

---

## 9. Logging

Log enough context to reproduce every error. Log nothing sensitive.

```ts
logger.error("API error", {
  url:       config.url,
  method:    config.method,
  status:    error.response?.status,
  errorCode: error.response?.data?.code,    // machine-readable code, not raw message
  requestId: error.response?.headers["x-request-id"],
  userId:    currentUser.id,                // no passwords, tokens, or PII beyond ID
});
```

Use `warn` for transient errors that resolved after retry.
Use `error` only for failures that reached the user.

---

## Quick reference

```
Receive API error
  ├─ 4xx (not 408/429) → surface to user, no retry
  │     ├─ 400 → inline validation message
  │     ├─ 401 → redirect to login (silent)
  │     ├─ 403 → access-denied message
  │     └─ 404 → not-found state
  └─ 408 | 429 | 5xx | network
        └─ retry ×3, exponential backoff + jitter
              ├─ success   → clear error state
              └─ exhausted → error fallback + "Try again" button
```