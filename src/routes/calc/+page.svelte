<script>
  import {
    auth,
    authFetch,
    onAuthSuccess,
    API_URL,
    getRole,
  } from "$lib/utils/auth.svelte.js";

  // ── calculator state ──────────────────────────────────────
  let display = $state("0");
  let expression = $state("");
  let justEvaled = $state(false);
  let memory = $state(0);
  let isScientific = $state(false);
  let angleMode = $state("deg"); // deg | rad

  // ── secret sequence state ─────────────────────────────────
  // Devin L-shape: 7, 4, 1, 2, 3 (down left col, across bottom)
  const DEVIN_SEQ = ["7", "4", "1", "2", "3"];
  // Gloss L-shape: 3, 6, 9, 8, 7 (reserved for later)
  const GLOSS_SEQ = ["3", "6", "9", "8", "7"];
  let keyHistory = $state([]);

  // ── admin panel state ─────────────────────────────────────
  let showAdmin = $state(false);
  let adminSpring = $state(false);
  let users = $state([]);
  let adminError = $state("");
  let loadingUsers = $state(false);
  let decodedJwt = $state(null);

  function decode(token) {
    try {
      return JSON.parse(atob(token.split(".")[1]));
    } catch {
      return null;
    }
  }

  // ── secret trigger ────────────────────────────────────────
  async function checkSequence(key) {
    keyHistory = [...keyHistory.slice(-4), key];
    const last5 = keyHistory.slice(-5);

    if (last5.join(",") === DEVIN_SEQ.join(",")) {
      if (!auth.accessToken) return;

      const res = await authFetch(`${API_URL}/auth/refresh`, {
        method: "POST",
      });

      if (!res.ok) return;

      const { access_token } = await res.json();
      onAuthSuccess(access_token);

      const payload = decode(access_token);
      const role = getRole(payload?.roles, "global");

      if (role !== "devin" && role !== "owen") return;
      decodedJwt = payload;
      triggerAdminSpring();
      await loadUsers();
      keyHistory = [];
    }
  }
  const defaultWins = {
    users: { x: 80, y: 140, w: 420, z: 100, rot: 3 },
    jwt: { x: 180, y: 160, w: 320, z: 101, rot: -4 },
  };

  function triggerAdminSpring() {
    showAdmin = true;
    openWindows = [];
    // stagger them in
    setTimeout(() => {
      openWindows = ["users"];
    }, 0);
    setTimeout(() => {
      openWindows = ["users", "jwt"];
    }, 120);
  }

  // ── calculator logic ──────────────────────────────────────
  function toRad(x) {
    return angleMode === "deg" ? (x * Math.PI) / 180 : x;
  }

  function press(val) {
    checkSequence(val);

    if (justEvaled && !isNaN(val)) {
      display = val;
      expression = "";
      justEvaled = false;
      return;
    }
    if (justEvaled) justEvaled = false;

    if (display === "0" && !isNaN(val)) {
      display = val;
    } else if (display === "Error") {
      display = val;
    } else {
      display += val;
    }
  }

  function pressOp(op) {
    justEvaled = false;
    expression = display + " " + op;
    display = "0";
  }

  function evaluate() {
    try {
      const expr = expression + " " + display;
      // safe eval via Function
      const result = Function(
        '"use strict"; return (' +
          expr.replace(/×/g, "*").replace(/÷/g, "/") +
          ")",
      )();
      display = String(parseFloat(result.toFixed(10)));
      expression = "";
      justEvaled = true;
    } catch {
      display = "Error";
      expression = "";
    }
  }

  function clear() {
    display = "0";
    expression = "";
    justEvaled = false;
  }
  function clearEntry() {
    display = "0";
  }
  function backspace() {
    if (display.length <= 1 || display === "Error") {
      display = "0";
      return;
    }
    display = display.slice(0, -1);
  }
  function negate() {
    display = display.startsWith("-") ? display.slice(1) : "-" + display;
  }
  function percent() {
    display = String(parseFloat(display) / 100);
  }

  function sciFunc(fn) {
    const x = parseFloat(display);
    let result;
    switch (fn) {
      case "sin":
        result = Math.sin(toRad(x));
        break;
      case "cos":
        result = Math.cos(toRad(x));
        break;
      case "tan":
        result = Math.tan(toRad(x));
        break;
      case "asin":
        result = Math.asin(x) * (angleMode === "deg" ? 180 / Math.PI : 1);
        break;
      case "acos":
        result = Math.acos(x) * (angleMode === "deg" ? 180 / Math.PI : 1);
        break;
      case "atan":
        result = Math.atan(x) * (angleMode === "deg" ? 180 / Math.PI : 1);
        break;
      case "log":
        result = Math.log10(x);
        break;
      case "ln":
        result = Math.log(x);
        break;
      case "sqrt":
        result = Math.sqrt(x);
        break;
      case "sq":
        result = x * x;
        break;
      case "cube":
        result = x * x * x;
        break;
      case "inv":
        result = 1 / x;
        break;
      case "exp":
        result = Math.exp(x);
        break;
      case "abs":
        result = Math.abs(x);
        break;
      case "pi":
        display = String(Math.PI);
        return;
      case "e":
        display = String(Math.E);
        return;
      case "fact":
        if (x < 0 || !Number.isInteger(x)) {
          display = "Error";
          return;
        }
        result = Array.from({ length: x }, (_, i) => i + 1).reduce(
          (a, b) => a * b,
          1,
        );
        break;
    }
    display = String(parseFloat(result.toFixed(10)));
    justEvaled = true;
  }

  function memStore() {
    memory = parseFloat(display);
  }
  function memRecall() {
    display = String(memory);
  }
  function memClear() {
    memory = 0;
  }
  function memAdd() {
    memory += parseFloat(display);
  }

  // keyboard support
  function onKey(e) {
    if (showAdmin) return;
    const k = e.key;
    if (!isNaN(k)) {
      press(k);
      return;
    }
    if (k === ".") press(".");
    if (k === "Enter" || k === "=") evaluate();
    if (k === "Backspace") backspace();
    if (k === "Escape") clear();
    if (k === "+") pressOp("+");
    if (k === "-") pressOp("-");
    if (k === "*") pressOp("×");
    if (k === "/") {
      e.preventDefault();
      pressOp("÷");
    }
    if (k === "%") percent();
  }

  // ── admin panel ───────────────────────────────────────────
  async function loadUsers() {
    loadingUsers = true;
    adminError = "";
    try {
      const res = await authFetch(`${API_URL}/auth/admin/users`);
      if (!res.ok) throw new Error("failed");
      users = await res.json();
    } catch {
      adminError = "couldn't load users";
    } finally {
      loadingUsers = false;
    }
  }

  //INFO: NEED TO IMPLEMENT THESE LATER

  async function deauthUser(userId) {
    await authFetch(`${API_URL}/auth/admin/revoke_all/${userId}`, {
      method: "DELETE",
    });
    await loadUsers();
  }

  /*  async function requestPasswordChange(userId) {
    await authFetch(
      `${API_URL}/auth/admin/users/${userId}/force-password-reset`,
      {
        method: "POST",
      },
    );
  } */

  async function evictUser(userId) {
    await authFetch(`${API_URL}/auth/admin/users/${userId}/evict`, {
      method: "POST",
    });
  }

  async function deleteUser(userId) {
    await authFetch(`${API_URL}/auth/admin/users/${userId}/delete`, {
      method: "DELETE",
    });
  }

  async function changeRole(userId, service, role) {
    await authFetch(`${API_URL}/auth/admin/users/${userId}/role`, {
      method: "PATCH",
      headers: { "Content-Type": "application/json" },
      body: JSON.stringify({ service, role }),
    });
    await loadUsers();
  }

  // ── window manager state ──────────────────────────────────
  let openWindows = $state([]);
  let selectedUser = $state("");
  let zCounter = $state(100);

  let wins = $state({ ...defaultWins });

  /*
//  for future
  $effect(() => {
    console.log(`Jwt X: ${wins.jwt.x}`);
    console.log(`Jwt Y: ${wins.jwt.y}`);
    console.log(`Users X: ${wins.users.x}`);
    console.log(`Users Y: ${wins.users.y}`);
  });
  */

  function closeWindow(id) {
    openWindows = openWindows.filter((w) => w !== id);
  }

  function bringToFront(id) {
    zCounter += 1;
    wins[id] = { ...wins[id], z: zCounter };
  }

  // ── drag logic ────────────────────────────────────────────
  let dragging = $state(null);
  let dragOffset = $state({ x: 0, y: 0 });

  function startDrag(e, id) {
    e.preventDefault();
    e.stopPropagation();
    bringToFront(id);
    dragging = id;
    dragOffset = {
      x: e.clientX - wins[id].x,
      y: e.clientY - wins[id].y,
    };
  }

  function onMouseMove(e) {
    if (!dragging) return;
    wins[dragging] = {
      ...wins[dragging],
      x: e.clientX - dragOffset.x,
      y: e.clientY - dragOffset.y,
    };
    wins = wins; // trigger reactivity
  }

  function onMouseUp() {
    dragging = null;
  }

  function onUserSelect() {}
</script>

<svelte:window
  onkeydown={onKey}
  onmousemove={onMouseMove}
  onmouseup={onMouseUp}
/>

<div class="page">
  <div class="calc-wrap">
    <!-- display -->
    <div class="display">
      <div class="expression">{expression}</div>
      <div class="number">{display}</div>
      {#if memory !== 0}
        <div class="mem-indicator">M: {memory}</div>
      {/if}
    </div>

    <!-- mode toggles -->
    <div class="mode-bar">
      <button
        class:active={!isScientific}
        onclick={() => (isScientific = false)}>basic</button
      >
      <button class:active={isScientific} onclick={() => (isScientific = true)}
        >scientific</button
      >
      {#if isScientific}
        <button
          class:active={angleMode === "deg"}
          onclick={() => (angleMode = "deg")}>deg</button
        >
        <button
          class:active={angleMode === "rad"}
          onclick={() => (angleMode = "rad")}>rad</button
        >
      {/if}
    </div>

    <!-- scientific rows -->
    {#if isScientific}
      <div class="sci-grid">
        <button onclick={() => sciFunc("sin")}>sin</button>
        <button onclick={() => sciFunc("cos")}>cos</button>
        <button onclick={() => sciFunc("tan")}>tan</button>
        <button onclick={() => sciFunc("asin")}>sin⁻¹</button>
        <button onclick={() => sciFunc("acos")}>cos⁻¹</button>
        <button onclick={() => sciFunc("atan")}>tan⁻¹</button>
        <button onclick={() => sciFunc("log")}>log</button>
        <button onclick={() => sciFunc("ln")}>ln</button>
        <button onclick={() => sciFunc("exp")}>eˣ</button>
        <button onclick={() => sciFunc("sqrt")}>√</button>
        <button onclick={() => sciFunc("sq")}>x²</button>
        <button onclick={() => sciFunc("cube")}>x³</button>
        <button onclick={() => sciFunc("inv")}>1/x</button>
        <button onclick={() => sciFunc("abs")}>|x|</button>
        <button onclick={() => sciFunc("fact")}>n!</button>
        <button onclick={() => sciFunc("pi")}>π</button>
        <button onclick={() => sciFunc("e")}>e</button>
        <button
          onclick={() => {
            press("(");
          }}>(</button
        >
        <button
          onclick={() => {
            press(")");
          }}>)</button
        >
        <button onclick={() => pressOp("**")}>xʸ</button>
      </div>
      <div class="mem-bar">
        <button onclick={memStore}>MS</button>
        <button onclick={memRecall}>MR</button>
        <button onclick={memClear}>MC</button>
        <button onclick={memAdd}>M+</button>
      </div>
    {/if}

    <!-- main grid -->
    <div class="grid">
      <button class="fn" onclick={clear}>AC</button>
      <button class="fn" onclick={clearEntry}>CE</button>
      <button class="fn" onclick={backspace}>⌫</button>
      <button class="op" onclick={() => pressOp("÷")}>÷</button>

      <button onclick={() => press("7")}>7</button>
      <button onclick={() => press("8")}>8</button>
      <button onclick={() => press("9")}>9</button>
      <button class="op" onclick={() => pressOp("×")}>×</button>

      <button onclick={() => press("4")}>4</button>
      <button onclick={() => press("5")}>5</button>
      <button onclick={() => press("6")}>6</button>
      <button class="op" onclick={() => pressOp("-")}>−</button>

      <button onclick={() => press("1")}>1</button>
      <button onclick={() => press("2")}>2</button>
      <button onclick={() => press("3")}>3</button>
      <button class="op" onclick={() => pressOp("+")}>+</button>

      <button onclick={negate}>+/−</button>
      <button onclick={() => press("0")}>0</button>
      <button onclick={() => press(".")}>.</button>
      <button class="eq" onclick={evaluate}>=</button>
    </div>
  </div>
</div>

<!-- ── ADMIN PANEL OVERLAY ── -->
<!-- ── ADMIN PANEL ── -->
{#if showAdmin}
  <button
    class="admin-close"
    onclick={() => {
      showAdmin = false;
      openWindows = [];
    }}>✕ close admin</button
  >

  {#each openWindows as winId, i (winId)}
    <div
      class="window"
      style="
        left:{wins[winId].x}px;
        top:{wins[winId].y}px;
        width:{wins[winId].w}px;
        z-index:{wins[winId].z};
        --delay:{i * 120}ms;
        --rot:{wins[winId].rot}deg;
      "
      onmousedown={() => bringToFront(winId)}
    >
      <div class="win-titlebar" onmousedown={(e) => startDrag(e, winId)}>
        <span>{winId}</span>
        <button onclick={() => closeWindow(winId)}>✕</button>
      </div>

      {#if winId === "users"}
        <div class="win-body">
          <div class="field-row">
            <label>user</label>
            <select bind:value={selectedUser}>
              <option value="">— select a user —</option>
              {#each users as u}
                <option value={u.id}>{u.username} ({u.id})</option>
              {/each}
            </select>
          </div>

          {#if selectedUser}
            {@const u = users.find((x) => x.id === selectedUser)}
            <div class="user-detail">
              <p class="section-label">roles</p>
              {#each ["global", "gradegetter", "gloss"] as svc}
                <div class="role-row">
                  <span class="svc-name">{svc}</span>
                  <select
                    value={u?.roles?.[svc] ?? "user"}
                    onchange={(e) =>
                      changeRole(selectedUser, svc, e.target.value)}
                  >
                    <option value="user">user</option>
                    <option value="trusted">trusted</option>
                    <option value="devin">devin</option>
                    <option value="owen">owen</option>
                  </select>
                </div>
              {/each}

              <p class="section-label" style="margin-top:1rem">actions</p>
              <div class="action-row">
                <button class="act-btn" onclick={() => evictUser(selectedUser)}>
                  evict</button
                >
                <button
                  class="act-btn"
                  onclick={() => deauthUser(selectedUser)}
                >
                  deauth</button
                >
                <button
                  class="act-btn"
                  onclick={() => deleteUser(selectedUser)}
                >
                  delete user</button
                >

                <!--><button
                  class="act-btn warn"
                  onclick={() => requestPasswordChange(selectedUser)}
                  > pw reset</button
                > <!-->
              </div>
            </div>
          {/if}

          {#if loadingUsers}
            <p class="muted">loading...</p>
          {:else if adminError}
            <p class="err">{adminError}</p>
          {/if}
          <button class="refresh-btn" onclick={loadUsers}>↻ refresh</button>
        </div>
      {/if}

      {#if winId === "jwt"}
        <div class="win-body">
          <div class="jwt-pill">
            <span class="role-badge"
              >{getRole(decodedJwt?.roles, "global")}</span
            >
            <span class="exp-badge"
              >exp {new Date(decodedJwt?.exp * 1000).toLocaleTimeString()}</span
            >
          </div>
          <pre>{JSON.stringify(decodedJwt, null, 2)}</pre>
        </div>
      {/if}
    </div>
  {/each}
{/if}

<style>
  .page {
    max-width: var(--column-width);
    margin: var(--column-margin-top) auto;
    padding: 0 1rem 4rem;
    display: flex;
    flex-direction: column;
    gap: 2rem;
    position: relative;
  }

  /* ── calculator ── */
  .calc-wrap {
    background: rgba(255, 255, 255, 0.02);
    border: 1px solid var(--color-border);
    border-radius: 1rem;
    padding: 1.5rem;
    backdrop-filter: blur(6px);
    box-shadow: 0 4px 12px rgba(0, 0, 0, 0.3);
    max-width: 360px;
    margin: 0 auto;
    width: 100%;
  }

  .display {
    background: rgba(0, 0, 0, 0.3);
    border: 1px solid var(--color-border);
    border-radius: 0.6rem;
    padding: 1rem;
    text-align: right;
    margin-bottom: 1rem;
    min-height: 80px;
    display: flex;
    flex-direction: column;
    justify-content: flex-end;
    position: relative;
  }
  .expression {
    font-size: 0.8rem;
    color: var(--color-subtle-text);
    font-family: var(--font-mono);
    min-height: 1.2rem;
  }
  .number {
    font-size: 2rem;
    font-weight: 600;
    color: var(--color-theme-1);
    font-family: var(--font-mono);
    word-break: break-all;
  }
  .mem-indicator {
    position: absolute;
    top: 0.5rem;
    left: 0.75rem;
    font-size: 0.7rem;
    color: var(--color-theme-2);
    font-family: var(--font-mono);
  }

  .mode-bar {
    display: flex;
    gap: 0.25rem;
    margin-bottom: 0.75rem;
  }
  .mode-bar button {
    flex: 1;
    padding: 0.3rem;
    font-size: 0.75rem;
    background: rgba(255, 255, 255, 0.03);
    border: 1px solid var(--color-border);
    border-radius: 0.35rem;
    color: var(--color-subtle-text);
    cursor: pointer;
    transition: all 0.15s ease;
  }
  .mode-bar button.active {
    background: rgba(126, 156, 255, 0.15);
    border-color: rgba(126, 156, 255, 0.4);
    color: var(--color-theme-1);
  }

  .sci-grid {
    display: grid;
    grid-template-columns: repeat(5, 1fr);
    gap: 0.35rem;
    margin-bottom: 0.5rem;
  }
  .sci-grid button {
    padding: 0.4rem 0.2rem;
    font-size: 0.72rem;
    font-family: var(--font-mono);
    background: rgba(154, 199, 255, 0.05);
    border: 1px solid var(--color-border);
    border-radius: 0.35rem;
    color: var(--color-theme-2);
    cursor: pointer;
    transition: background 0.15s ease;
  }
  .sci-grid button:hover {
    background: rgba(154, 199, 255, 0.12);
  }

  .mem-bar {
    display: grid;
    grid-template-columns: repeat(4, 1fr);
    gap: 0.35rem;
    margin-bottom: 0.5rem;
  }
  .mem-bar button {
    padding: 0.35rem;
    font-size: 0.72rem;
    font-family: var(--font-mono);
    background: rgba(255, 255, 255, 0.03);
    border: 1px solid var(--color-border);
    border-radius: 0.35rem;
    color: var(--color-subtle-text);
    cursor: pointer;
    transition: background 0.15s ease;
  }
  .mem-bar button:hover {
    background: rgba(255, 255, 255, 0.08);
  }

  .grid {
    display: grid;
    grid-template-columns: repeat(4, 1fr);
    gap: 0.5rem;
  }
  .grid button {
    padding: 1rem 0.5rem;
    font-size: 1.1rem;
    font-family: var(--font-mono);
    background: rgba(255, 255, 255, 0.04);
    border: 1px solid var(--color-border);
    border-radius: 0.5rem;
    color: var(--color-text);
    cursor: pointer;
    transition:
      background 0.15s ease,
      transform 0.08s ease;
  }
  .grid button:hover {
    background: rgba(255, 255, 255, 0.1);
  }
  .grid button:active {
    transform: scale(0.93);
  }
  .grid button.fn {
    color: var(--color-subtle-text);
    font-size: 0.9rem;
  }
  .grid button.op {
    color: var(--color-theme-2);
    background: rgba(154, 199, 255, 0.06);
  }
  .grid button.op:hover {
    background: rgba(154, 199, 255, 0.14);
  }
  .grid button.eq {
    background: rgba(126, 156, 255, 0.2);
    border-color: rgba(126, 156, 255, 0.4);
    color: var(--color-theme-1);
    font-weight: 700;
  }
  .grid button.eq:hover {
    background: rgba(126, 156, 255, 0.35);
  }

  /* ── admin close button ── */
  .admin-close {
    position: fixed;
    top: 3.5rem;
    right: 1rem;
    z-index: 999;
    background: rgba(255, 100, 100, 0.1);
    border: 1px solid rgba(255, 100, 100, 0.3);
    color: #ff9090;
    border-radius: 0.4rem;
    padding: 0.35rem 0.9rem;
    font-size: 0.8rem;
    cursor: pointer;
    transition: background 0.15s;
    animation: windowSpringIn 0.6s cubic-bezier(0.175, 0.885, 0.32, 1.6)
      forwards;
    --rot: -2deg;
  }
  .admin-close:hover {
    background: rgba(255, 100, 100, 0.2);
  }

  /* ── windows ── */
  .window {
    position: fixed;
    background: rgba(20, 10, 35, 0.97);
    border: 1px solid var(--color-border);
    border-radius: 0.6rem;
    box-shadow: 0 8px 32px rgba(0, 0, 0, 0.7);
    min-width: 280px;
    user-select: none;
    backdrop-filter: blur(12px);
    animation: windowSpringIn 0.65s cubic-bezier(0.175, 0.885, 0.32, 1.8)
      var(--delay, 0ms) both;
  }

  @keyframes windowSpringIn {
    0% {
      transform: scale(0) rotate(calc(var(--rot) * -4)) translateY(300px);
      opacity: 0;
    }
    25% {
      opacity: 1;
    }
    35% {
      transform: scale(1.18) rotate(calc(var(--rot) * 1.5)) translateY(-40px);
    }
    55% {
      transform: scale(0.91) rotate(calc(var(--rot) * -0.8)) translateY(18px);
    }
    70% {
      transform: scale(1.07) rotate(calc(var(--rot) * 0.5)) translateY(-12px);
    }
    82% {
      transform: scale(0.96) rotate(calc(var(--rot) * -0.2)) translateY(6px);
    }
    91% {
      transform: scale(1.02) rotate(calc(var(--rot) * 0.1)) translateY(-3px);
    }
    100% {
      transform: scale(1) rotate(var(--rot)) translateY(0);
      opacity: 1;
    }
  }

  .win-titlebar {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 0.4rem 0.75rem;
    background: rgba(255, 255, 255, 0.04);
    border-bottom: 1px solid var(--color-border);
    border-radius: 0.6rem 0.6rem 0 0;
    cursor: grab;
    font-size: 0.8rem;
    font-weight: 600;
    color: var(--color-theme-2);
  }
  .win-titlebar:active {
    cursor: grabbing;
  }
  .win-titlebar button {
    background: none;
    border: none;
    color: var(--color-subtle-text);
    cursor: pointer;
    font-size: 0.8rem;
    padding: 0 0.25rem;
    line-height: 1;
  }
  .win-titlebar button:hover {
    color: #ff9090;
    background: none;
  }

  .win-body {
    padding: 1rem;
    display: flex;
    flex-direction: column;
    gap: 0.75rem;
    max-height: 420px;
    overflow-y: auto;
  }

  /* ── users window ── */
  .field-row {
    display: flex;
    flex-direction: column;
    gap: 0.35rem;
  }
  .field-row label {
    font-size: 0.75rem;
    color: var(--color-subtle-text);
    text-transform: uppercase;
    letter-spacing: 0.08em;
  }
  .field-row select {
    width: 100%;
    font-size: 0.85rem;
    padding: 0.4rem 0.6rem;
    background: rgba(255, 255, 255, 0.05);
    border: 1px solid var(--color-border);
    border-radius: 0.4rem;
    color: var(--color-text);
  }
  .user-detail {
    display: flex;
    flex-direction: column;
    gap: 0.5rem;
  }
  .role-row {
    display: flex;
    align-items: center;
    gap: 0.75rem;
  }
  .svc-name {
    font-size: 0.8rem;
    font-family: var(--font-mono);
    color: var(--color-subtle-text);
    min-width: 80px;
  }
  .role-row select {
    flex: 1;
    font-size: 0.8rem;
    padding: 0.3rem 0.5rem;
    background: rgba(255, 255, 255, 0.05);
    border: 1px solid var(--color-border);
    border-radius: 0.35rem;
    color: var(--color-text);
  }
  .action-row {
    display: flex;
    flex-wrap: wrap;
    gap: 0.4rem;
  }
  .act-btn {
    font-size: 0.75rem;
    padding: 0.3rem 0.6rem;
    background: rgba(255, 255, 255, 0.04);
    border: 1px solid var(--color-border);
    border-radius: 0.35rem;
    color: var(--color-subtle-text);
    cursor: pointer;
    transition: background 0.15s;
    white-space: nowrap;
  }
  .act-btn:hover {
    background: rgba(255, 255, 255, 0.1);
    color: var(--color-text);
  }
  .act-btn.warn {
    color: #ff9090;
    border-color: rgba(255, 100, 100, 0.25);
  }
  .act-btn.warn:hover {
    background: rgba(255, 100, 100, 0.1);
  }
  .refresh-btn {
    font-size: 0.75rem;
    padding: 0.25rem 0.6rem;
    background: rgba(255, 255, 255, 0.04);
    border: 1px solid var(--color-border);
    border-radius: 0.35rem;
    color: var(--color-subtle-text);
    cursor: pointer;
    align-self: flex-start;
    transition: background 0.15s;
  }
  .refresh-btn:hover {
    background: rgba(255, 255, 255, 0.1);
  }

  /* ── jwt window ── */
  .jwt-pill {
    display: flex;
    gap: 0.5rem;
    align-items: center;
    flex-wrap: wrap;
  }
  .role-badge {
    font-size: 0.75rem;
    font-family: var(--font-mono);
    background: rgba(126, 156, 255, 0.15);
    border: 1px solid rgba(126, 156, 255, 0.3);
    color: var(--color-theme-1);
    padding: 2px 8px;
    border-radius: 0.3rem;
  }
  .exp-badge {
    font-size: 0.75rem;
    font-family: var(--font-mono);
    color: var(--color-subtle-text);
  }
  .win-body pre {
    font-size: 0.72rem;
    margin: 0;
    max-height: 300px;
    overflow-y: auto;
  }

  /* ── shared ── */
  .section-label {
    font-size: 0.7rem;
    text-transform: uppercase;
    letter-spacing: 0.1em;
    color: var(--color-subtle-text);
    margin: 0;
  }
  .muted {
    font-size: 0.8rem;
    color: var(--color-subtle-text);
    margin: 0;
  }
  .err {
    font-size: 0.8rem;
    color: #ff9090;
    margin: 0;
  }
  .close-btn {
    background: rgba(255, 100, 100, 0.08);
    border: 1px solid rgba(255, 100, 100, 0.25);
    color: #ff9090;
    border-radius: 0.4rem;
    padding: 0.3rem 0.75rem;
    font-size: 0.8rem;
    cursor: pointer;
    transition: background 0.15s;
  }
  .close-btn:hover {
    background: rgba(255, 100, 100, 0.18);
  }
</style>
