<script>
  import { onMount } from "svelte";
  import {
    authFetch,
    API_URL,
    auth,
    logout,
    getRole,
  } from "$lib/utils/auth.svelte.js";
  import { goto } from "$app/navigation";

  let sessions = $state([]);
  let confirmText = $state("");
  let showConfirm = $state(false);

  function parseUA(ua = "") {
    if (!ua)
      return { browser: "Unknown browser", os: "Unknown OS", icon: "desktop" };
    const browser = ua.includes("Firefox")
      ? "Firefox"
      : ua.includes("Edg")
        ? "Edge"
        : ua.includes("Chrome")
          ? "Chrome"
          : ua.includes("Safari")
            ? "Safari"
            : "Browser";

    const os =
      ua.includes("iPhone") || ua.includes("iPad")
        ? "iOS"
        : ua.includes("Android")
          ? "Android"
          : ua.includes("CrOS")
            ? "ChromeOS"
            : ua.includes("Mac OS")
              ? "macOS"
              : ua.includes("Windows")
                ? "Windows"
                : ua.includes("Linux")
                  ? "Linux"
                  : "Unknown OS";
    const icon = os === "iOS" || os === "Android" ? "mobile" : "desktop";
    return { browser, os, icon };
  }

  function expiresIn(unix) {
    const diff = unix - Math.floor(Date.now() / 1000);
    const days = Math.floor(diff / 86400);
    if (days > 1) return `expires in ${days} days`;
    if (days === 1) return "expires tomorrow";
    return "expires today";
  }

  async function fetch_sessions() {
    const res = await authFetch(`${API_URL}/auth/sessions/list_all`);
    if (res.ok) sessions = await res.json();
  }

  $effect(() => {
    if (!auth.ready) return;

    fetch_sessions();
  });

  onMount(async () => {});

  async function revoke(id) {
    const res = await authFetch(`${API_URL}/auth/sessions/revoke/${id}`, {
      method: "DELETE",
    });
    if (res.ok) sessions = sessions.filter((s) => s.session_id !== id);
  }

  async function revokeAll() {
    const res = await authFetch(`${API_URL}/auth/sessions/revoke`, {
      method: "DELETE",
    });
    if (res.ok) sessions = sessions.filter((s) => s.is_current);
  }

  async function deleteAccount() {
    if (confirmText.toLowerCase() !== "delete") return;
    const res = await authFetch(`${API_URL}/auth/delete`, {
      method: "DELETE",
    });
    if (res.ok) {
      logout();
      goto("/");
    }
  }

  let schEmail = $state("");
  let schPassword = $state("");
  let schErr = $state("");
  let wsStatus = $state(null); // null = idle, "running", "done", "error"
  let wsSteps = $state([]);
  let wsProgress = $state(0);

  const STEPS = [
    "Started",
    "Navigated to Schoology login",
    "Typed in Email",
    "Entered Email",
    "Typed in Password",
    "Enter Password",
    "Finished",
  ];

  async function addSchoology() {
    schErr = "";
    if (!schEmail || !schPassword) {
      schErr = "fill in all fields";
      return;
    }

    const res = await authFetch(
      `${API_URL}/gradegetter/auth/schoology/credentials`,
      {
        method: "POST",
        headers: { "Content-Type": "application/json" },
        credentials: "include",
        body: JSON.stringify({
          schoology_email: schEmail,
          schoology_password: schPassword,
        }),
      },
    );

    if (!res.ok) {
      schErr =
        res.status === 409 ? "username already taken" : "something went wrong";
      return;
    }

    await authFetch(`${API_URL}/gradegetter/auth/forward`, {
      method: "GET",
    });

    const wsUrl = `${API_URL.replace("https://", "wss://").replace("http://", "ws://")}/gradegetter/auth/forward_ws/${auth.id}`;
    //const wsUrl = `${API_URL.replace("https://", "wss://")}/gradegetter/auth/forward_ws/${auth.id}`;
    const socket = new WebSocket(wsUrl);

    socket.onmessage = (e) => {
      const raw = e.data.trim();
      const [label, numStr] = raw.split(",");
      const num = parseInt(numStr);

      if (label.startsWith("ERROR")) {
        wsStatus = "error";
        schErr = label;
        socket.close();
        return;
      }

      wsSteps = [...wsSteps, label];
      wsProgress = Math.round((num / 7) * 100);

      if (num >= 7) {
        wsStatus = "done";
        socket.close();
      }
    };

    socket.onerror = () => {
      wsStatus = "error";
      schErr = "websocket connection failed";
    };

    socket.onclose = () => {
      if (wsStatus === "running") wsStatus = "error";
    };
  }

  async function deleteSchoology() {
    await authFetch(`${API_URL}/gradegetter/auth/schoology/credentials`, {
      method: "DELETE",
    });
  }
</script>

<svelte:head>
  <title>DevinLittle.net - Settings</title>
  <meta name="description" content="The settings page of devinlittle.net" />
</svelte:head>

{#if auth.ready}
  <div class="wrap">
    <h1>Settings</h1>

    <div class="card">
      <p class="card-title">Active sessions</p>
      <p class="card-sub">All devices currently signed in to your account.</p>

      {#each sessions as session}
        {@const { browser, os, icon } = parseUA(session.user_agent)}
        <div class="session-row">
          <div class="session-icon">
            {#if icon === "mobile"}
              <!-- mobile svg -->
              <svg
                xmlns="http://www.w3.org/2000/svg"
                width="18"
                height="18"
                viewBox="0 0 24 24"
                fill="none"
                stroke="currentColor"
                stroke-width="1.5"
              >
                <rect x="5" y="2" width="14" height="20" rx="2" /><circle
                  cx="12"
                  cy="18"
                  r="1"
                />
              </svg>
            {:else}
              <!-- desktop svg -->
              <svg
                xmlns="http://www.w3.org/2000/svg"
                width="18"
                height="18"
                viewBox="0 0 24 24"
                fill="none"
                stroke="currentColor"
                stroke-width="1.5"
              >
                <rect x="2" y="3" width="20" height="14" rx="2" /><path
                  d="M8 21h8M12 17v4"
                />
              </svg>
            {/if}
          </div>
          <div class="session-info">
            <div class="session-name">
              {browser} on {os}
              {#if session.is_current}<span class="badge">current</span>{/if}
            </div>
            <div class="session-meta">{expiresIn(session.expires_at)}</div>
          </div>
          <!--> {#if !session.is_current} <!-->
          <button
            class="btn btn-danger"
            onclick={() => revoke(session.session_id)}>Revoke</button
          >
          <!--> {/if} <!-->
        </div>
      {/each}

      <button class="btn btn-danger btn-full" onclick={revokeAll}>
        Sign out of all other sessions
      </button>
    </div>

    <div class="card">
      <p class="card-title">Delete account</p>
      <p class="card-sub">Permanently remove your account and all data.</p>
      <div class="warn">This cannot be undone.</div>

      {#if !showConfirm}
        <button class="btn btn-danger" onclick={() => (showConfirm = true)}
          >Delete my account</button
        >
      {:else}
        <p class="confirm-label">Type <strong>delete</strong> to confirm</p>
        <input type="text" bind:value={confirmText} placeholder="delete" />
        <div class="confirm-row">
          <button class="btn btn-danger" onclick={deleteAccount}
            >Yes, delete my account</button
          >
          <button
            class="btn"
            onclick={() => {
              showConfirm = false;
              confirmText = "";
            }}>Cancel</button
          >
        </div>
      {/if}
    </div>
  </div>

  <div class="wrap">
    {#if ["trusted", "devin", "owen"].includes(getRole(auth.roles, "gradegetter"))}
      <div class="card">
        <p class="card-title">Schoology Information</p>

        {#if wsStatus === null || wsStatus === "done" || wsStatus === "error"}
          <div class="form">
            <input type="text" placeholder="email" bind:value={schEmail} />
            <input
              type="password"
              placeholder="password"
              bind:value={schPassword}
              onkeydown={(e) => e.key === "Enter" && addSchoology()}
            />
            {#if schErr}<p class="error">{schErr}</p>{/if}
            <button onclick={addSchoology} disabled={wsStatus === "running"}>
              Add Information
            </button>
          </div>
        {/if}

        {#if wsStatus !== null}
          <div class="ws-progress">
            <!-- bar -->
            <div class="progress-track">
              <div
                class="progress-fill"
                class:done={wsStatus === "done"}
                class:error={wsStatus === "error"}
                style="width: {wsProgress}%"
              ></div>
            </div>

            <!-- percent + status -->
            <div class="progress-meta">
              <span class="progress-pct">{wsProgress}%</span>
              <span
                class="progress-state"
                class:done={wsStatus === "done"}
                class:error={wsStatus === "error"}
              >
                {#if wsStatus === "running"}logging in...
                {:else if wsStatus === "done"}done!
                {:else}failed{/if}
              </span>
            </div>

            <!-- step log -->
            <div class="step-log">
              {#each wsSteps as step, i}
                <div class="step-item" class:latest={i === wsSteps.length - 1}>
                  <span class="step-dot"></span>
                  <span>{step}</span>
                </div>
              {/each}
              {#if wsStatus === "running"}
                <div class="step-item pending">
                  <span class="step-dot pulse"></span>
                  <span>{STEPS[wsSteps.length] ?? "..."}</span>
                </div>
              {/if}
            </div>

            {#if wsStatus === "done" || wsStatus === "error"}
              <button
                class="btn-reset"
                onclick={() => {
                  wsStatus = null;
                  wsSteps = [];
                  wsProgress = 0;
                  schErr = "";
                }}
              >
                ↩ reset
              </button>
            {/if}
          </div>
        {/if}

        <button class="btn btn-danger" onclick={deleteSchoology}>
          Delete schoology information from my account
        </button>
      </div>
    {/if}
  </div>
{:else}
  <h1>Login to View Settings</h1>
{/if}

<style>
  .wrap {
    max-width: var(--column-width);
    margin: var(--column-margin-top) auto;
    padding: 0 1rem;
    display: flex;
    flex-direction: column;
    gap: 1.5rem;
  }
  .card {
    background: rgba(255, 255, 255, 0.02);
    border: 1px solid var(--color-border);
    border-radius: 1rem;
    padding: 1.5rem;
    backdrop-filter: blur(6px);
    box-shadow: 0 4px 12px rgba(0, 0, 0, 0.3);
  }
  .card-title {
    font-size: 1.4rem;
    font-weight: 500;
    color: var(--color-theme-2);
    border-bottom: 1px solid var(--color-border);
    padding-bottom: 0.5rem;
    margin: 0 0 0.5rem;
  }
  .card-sub {
    font-size: 0.875rem;
    color: var(--color-subtle-text);
    margin: 0 0 1rem;
  }
  .session-row {
    display: flex;
    align-items: center;
    gap: 0.75rem;
    background: rgba(255, 255, 255, 0.04);
    padding: 0.75rem 1rem;
    border-radius: 0.5rem;
    margin-bottom: 0.5rem;
    transition: background 0.2s ease;
  }
  .session-row:hover {
    background: rgba(255, 255, 255, 0.07);
  }
  .session-icon {
    width: 36px;
    height: 36px;
    border-radius: 0.4rem;
    background: rgba(255, 255, 255, 0.05);
    border: 1px solid var(--color-border);
    display: flex;
    align-items: center;
    justify-content: center;
    flex-shrink: 0;
    color: var(--color-subtle-text);
  }
  .session-info {
    flex: 1;
    min-width: 0;
  }
  .session-name {
    font-size: 0.9rem;
    font-weight: 500;
    color: var(--color-text);
    display: flex;
    align-items: center;
    gap: 0.5rem;
  }
  .session-meta {
    font-size: 0.8rem;
    color: var(--color-subtle-text);
    margin-top: 2px;
  }
  .badge {
    font-size: 0.7rem;
    padding: 1px 7px;
    border-radius: 0.3rem;
    background: rgba(126, 156, 255, 0.15);
    color: var(--color-theme-1);
    border: 1px solid rgba(126, 156, 255, 0.3);
  }
  .btn-danger {
    border-color: rgba(255, 100, 100, 0.3);
    color: #ff9090;
  }
  .btn-danger:hover {
    background: rgba(255, 100, 100, 0.1);
  }
  .btn-full {
    width: 100%;
    margin-top: 0.75rem;
  }
  .warn {
    background: rgba(255, 100, 100, 0.07);
    border: 1px solid rgba(255, 100, 100, 0.2);
    border-radius: 0.5rem;
    padding: 0.75rem 1rem;
    font-size: 0.85rem;
    color: #ff9090;
    margin-bottom: 1rem;
  }
  .ws-progress {
    display: flex;
    flex-direction: column;
    gap: 0.6rem;
    margin: 0.5rem 0;
  }
  .progress-track {
    width: 100%;
    height: 6px;
    background: rgba(255, 255, 255, 0.07);
    border-radius: 3px;
    overflow: hidden;
    border: 1px solid var(--color-border);
  }
  .progress-fill {
    height: 100%;
    background: var(--color-theme-1);
    border-radius: 3px;
    transition: width 0.4s ease;
  }
  .progress-fill.done {
    background: #6fcf97;
  }
  .progress-fill.error {
    background: #ff9090;
  }

  .progress-meta {
    display: flex;
    justify-content: space-between;
    align-items: center;
    font-size: 0.8rem;
  }
  .progress-pct {
    font-family: var(--font-mono);
    color: var(--color-theme-1);
  }
  .progress-state {
    color: var(--color-subtle-text);
  }
  .progress-state.done {
    color: #6fcf97;
  }
  .progress-state.error {
    color: #ff9090;
  }

  .step-log {
    display: flex;
    flex-direction: column;
    gap: 0.3rem;
    background: rgba(0, 0, 0, 0.2);
    border: 1px solid var(--color-border);
    border-radius: 0.5rem;
    padding: 0.75rem;
    max-height: 180px;
    overflow-y: auto;
  }
  .step-item {
    display: flex;
    align-items: center;
    gap: 0.5rem;
    font-size: 0.8rem;
    color: var(--color-subtle-text);
    opacity: 0.6;
    transition: opacity 0.2s;
  }
  .step-item.latest {
    opacity: 1;
    color: var(--color-text);
  }
  .step-item.pending {
    opacity: 0.4;
    font-style: italic;
  }
  .step-dot {
    width: 6px;
    height: 6px;
    border-radius: 50%;
    background: var(--color-theme-1);
    flex-shrink: 0;
  }
  .step-dot.pulse {
    animation: pulse 1s ease-in-out infinite;
  }
  @keyframes pulse {
    0%,
    100% {
      opacity: 1;
      transform: scale(1);
    }
    50% {
      opacity: 0.3;
      transform: scale(0.7);
    }
  }

  .btn-reset {
    font-size: 0.8rem;
    padding: 0.3rem 0.75rem;
    background: rgba(255, 255, 255, 0.04);
    border: 1px solid var(--color-border);
    border-radius: 0.4rem;
    color: var(--color-subtle-text);
    cursor: pointer;
    align-self: flex-start;
    transition: background 0.15s;
  }
  .btn-reset:hover {
    background: rgba(255, 255, 255, 0.1);
  }
  .confirm-box {
    display: flex;
    flex-direction: column;
    gap: 0.75rem;
    margin-top: 1rem;
    padding-top: 1rem;
    border-top: 1px solid var(--color-border);
  }
  .confirm-row {
    display: flex;
    gap: 0.5rem;
  }
  .revoked {
    opacity: 0.35;
    pointer-events: none;
  }
</style>
