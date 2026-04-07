<script>
  import { onMount } from "svelte";
  import { authFetch, API_URL, auth, logout } from "$lib/utils/auth.svelte.js";
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
    if (res.ok) sessions = sessions.filter((s) => s.id !== id);
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
</script>

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
          {#if !session.is_current}
            <button
              class="btn btn-danger"
              onclick={() => revoke(session.session_id)}>Revoke</button
            >
          {/if}
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
