(() => {
  "use strict";
  const state = { delay: 250, maxDelay: 5000, timer: null };
  const log = (...args) => console.debug("[Rust Live Server]", ...args);

  function samePath(link, changedPath) {
    try {
      return new URL(link.href, location.href).pathname === changedPath;
    } catch {
      return false;
    }
  }

  function hotCss(path) {
    const links = [
      ...document.querySelectorAll('link[rel="stylesheet"]'),
    ].filter((link) => samePath(link, path));
    if (!links.length) return location.reload();
    const stamp = Date.now();
    for (const link of links) {
      const url = new URL(link.href, location.href);
      url.searchParams.set("__live_server", String(stamp));

      const clone = link.cloneNode(false);
      clone.href = url.href;
      clone.onload = () => link.remove(); // Remove old stylesheet only after new one loads (prevents FOUC)

      link.parentNode.insertBefore(clone, link.nextSibling);
    }
    log("CSS hot reload", path);
  }

  function reload(event) {
    // Handle both object events and plain-text commands (e.g. "reload")
    const kind = typeof event === "string" ? event : event?.kind;
    const path = typeof event === "object" ? event?.path : location.pathname;

    if (!kind || kind === "css") {
      hotCss(path || location.pathname);
    } else {
      location.reload();
    }
  }

  function connect() {
    const scheme = location.protocol === "https:" ? "wss:" : "ws:";
    const ws = new WebSocket(`${scheme}//${location.host}/__live_server/ws`);

    ws.addEventListener("open", () => {
      state.delay = 250;
      log("connected");
    });

    ws.addEventListener("message", (message) => {
      let eventData;
      try {
        eventData = JSON.parse(message.data);
      } catch {
        eventData = message.data; // Fallback for plain-text messages
      }
      reload(eventData);
    });

    ws.addEventListener("close", () => {
      clearTimeout(state.timer);
      state.timer = setTimeout(connect, state.delay);
      state.delay = Math.min(state.delay * 2, state.maxDelay);
    });

    ws.addEventListener("error", () => ws.close());
  }

  connect();
})();
