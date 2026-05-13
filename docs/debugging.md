# Debugging

## Server-Side (Rust/Fullstack)

### Prerequisites

**Once on the host system** (not inside the container):

```bash
sudo sysctl -w kernel.yama.ptrace_scope=0
```

To make it persistent:

```bash
echo 'kernel.yama.ptrace_scope=0' | sudo tee /etc/sysctl.d/10-ptrace.conf
```

Inside the container `ptrace_scope` is already 0 — the devcontainer sets `SYS_PTRACE` + `seccomp:unconfined`.

### Zed

1. Start `just serve` and wait until it prints that the server is listening.
2. Set your breakpoints **before** attaching (Zed resolves pending breakpoints on attach).
3. Press `F4` in Zed → choose the **"Attach"** tab (not "Run").
4. Select the `server-<hash>` process from the list (e.g. `server-387bbe4a`). The binary lives under `zeitrak-presentation/gui/target/dx/web/debug/web/`.
5. Trigger the code path you want to break in — the debugger will stop at your breakpoint.

> **Why the hash suffix?** `dx serve` compiles a new binary for every hot-reload. The binary is always named `server-<8-char-hex>`. Pick the most recently created one from the process list.

> **After a hot-reload**, `dx serve` kills the old process and starts a new one. You must re-attach (repeat steps 3–4) each time.

### Server functions vs. client routes

Dioxus fullstack separates concerns into two layers:

| Layer | Where it runs | How to debug |
|---|---|---|
| **Client route** (`/verify-email/<token>`) | Browser (WASM) | Browser DevTools → Sources |
| **Server function** (`#[get("/api/verify-email")]`) | Rust server binary | LLDB attach (above) |

When you open `http://localhost:8080/verify-email/<token>` in the browser:

1. The server sends the SSR'd HTML + WASM bundle.
2. The WASM hydrates and the route component mounts.
3. The component calls `verify_email(token)` which POSTs to `/api/verify-email`.
4. **Only step 3 hits your server-side breakpoint** in `registration/mod.rs`.

So a direct `curl http://localhost:8080/verify-email/...` will **not** trigger the server function — the WASM must run first.

### Client-Side (WASM)

The WASM client runs in the browser and is debugged via browser DevTools.

1. Open Chrome/Firefox DevTools → "Sources" tab.
2. `.rs` files are visible directly thanks to source maps.
3. Breakpoints and step-through work with some limitations.

For readable Rust symbols in the browser, install the [DWARF Symbols Extension](https://chromewebstore.google.com/detail/cc++-devtools-support-dwa/pdcpmagijalfljmkmjngeonclgbbannb) for Chrome.

## Logging

The Dioxus logger is integrated — `tracing` macros work across all targets:

```rust
tracing::info!("server started");
tracing::debug!("request: {:?}", req);
```

Server logs appear in the `just serve` terminal, client logs in the browser console.
