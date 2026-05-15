<script lang="ts">
  import { onMount, tick } from "svelte";
  import { auth, logout, getRole, authApi } from "$lib/utils/auth.svelte";
  import { goto } from "$app/navigation";
  import {
    db_state,
    generate_and_store_keypair,
  } from "$lib/utils/sqlite.svelte";

  import {
    keysync,
    requestKeySync,
    respondToChallenge,
    cancelKeySync,
    generate_recovery_words,
    setup_recovery_phrase,
    recover_with_phrase,
  } from "$lib/utils/smalltalk.svelte";
  import {
    addNotification,
    notificationApi,
  } from "$lib/utils/notifications.svelte";
  import { gradesApi } from "$lib/utils/gradegetter.svelte";
  import { API_URL, VAPID_PUBLIC_KEY } from "$lib/utils/constants.svelte";

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
    let get_sessions = authApi.path("/me/sessions").method("get").create();
    let res = await get_sessions({});
    if (res.ok) sessions = await res.data;
  }

  onMount(async () => {});

  async function revoke(id) {
    let revokeSessionReq = authApi
      .path("/me/session/{id}")
      .method("delete")
      .create();
    let res = await revokeSessionReq({ id });
    if (res.ok) sessions = sessions.filter((s) => s.session_id !== id);
  }

  async function revokeAll() {
    let revokeAllSessions = authApi
      .path("/me/sessions")
      .method("delete")
      .create();
    let res = await revokeAllSessions({});
    if (res.ok) sessions = sessions.filter((s) => s.is_current);
  }

  async function deleteAccount() {
    if (confirmText.toLowerCase() !== "delete") return;

    let deleteAccReq = authApi.path("/me").method("delete").create();
    let res = await deleteAccReq({});

    if (res.ok) {
      logout();
      goto("/");
    }
  }

  async function startOnboarding() {
    await generate_and_store_keypair();
    addNotification({
      type: "important_info",
      title: "Notification",
      body: "Encryption Key added! Refresh the page for changes to take place.",
      sender: "DevinLittle.Net",
      global: false,
    });
  }

  let show_recovery_setup = $state(false);
  let show_recovery_recover = $state(false);
  let recovery_words = $state([]);
  let recovery_input = $state(Array(12).fill(""));
  let recovery_confirmed = $state(false);
  let recovery_loading = $state(false);
  let recovery_error = $state("");
  let recovery_success = $state(false);

  function startRecoverySetup() {
    recovery_words = generate_recovery_words();
    show_recovery_setup = true;
    recovery_confirmed = false;
  }

  async function confirmRecoverySetup() {
    recovery_loading = true;
    try {
      await setup_recovery_phrase(recovery_words);
      show_recovery_setup = false;
      recovery_words = [];
      recovery_confirmed = false;
    } finally {
      recovery_loading = false;
    }
  }

  async function attemptRecovery() {
    recovery_loading = true;
    recovery_error = "";
    try {
      const ok = await recover_with_phrase(recovery_input);
      if (ok) {
        recovery_success = true;
        show_recovery_recover = false;
        addNotification({
          type: "important_info",
          title: "Notification",
          body: "Encryption Key added! Refresh the page for changes to take place.",
          sender: "DevinLittle.Net",
          global: false,
        });
      } else {
        recovery_error = "incorrect recovery phrase, please try again";
      }
    } finally {
      recovery_loading = false;
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

    let addSchoologyCreds = gradesApi
      .path("/auth/schoology/credentials")
      .method("post")
      .create();
    let res = await addSchoologyCreds({
      schoology_email: schEmail,
      schoology_password: schPassword,
    });

    if (!res.ok) {
      schErr =
        res.status === 409 ? "username already taken" : "something went wrong";
      return;
    }

    let forwardReq = gradesApi.path("/auth/forward").method("get").create();
    await forwardReq({});

    wsStatus = "running";
    wsSteps = [];
    wsProgress = 0;

    const wsUrl = `${API_URL.replace("https://", "wss://").replace("http://", "ws://")}/gradegetter/auth/forward_ws/${auth.id}`;
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
    let delSchoology = gradesApi
      .path("/auth/schoology/credentials")
      .method("delete")
      .create();
    await delSchoology({});
  }

  type PermissionState = "granted" | "denied" | "default" | "unsupported";
  let notificationPermission: PermissionState = $state("default");
  let notificationSubscribed = $state(false);
  let notificationLoading = $state(false);
  let notificationError = $state("");

  function urlBase64ToUint8Array(base64String: string) {
    const padding = "=".repeat((4 - (base64String.length % 4)) % 4);
    const base64 = (base64String + padding)
      .replace(/-/g, "+")
      .replace(/_/g, "/");
    const raw = atob(base64);
    return Uint8Array.from([...raw].map((c) => c.charCodeAt(0)));
  }

  async function postSubscription(subscription: PushSubscription) {
    const subscriptonJson = subscription.toJSON();
    const keys = subscriptonJson.keys as { auth: string; p256dh: string };
    let subsctibeReq = notificationApi
      .path("/subscribe")
      .method("post")
      .create();
    let res = await subsctibeReq({
      endpoint: subscriptonJson.endpoint,
      keys,
    });
    if (!res.ok) throw new Error(`Server error: ${res.status}`);
  }

  async function enableNotifications() {
    notificationError = "";
    notificationLoading = true;

    try {
      // 1. Force the permission prompt (Chrome requirement for user gesture)
      const permission = await Notification.requestPermission();
      if (permission !== "granted") {
        throw new Error(
          "Permission denied. Please enable notifications in your browser settings.",
        );
      }

      // 2. Wait for the Service Worker
      const reg = await navigator.serviceWorker.ready;

      // 3. CLEANUP: Unsubscribe any existing subscription to prevent "hanging"
      const existingSub = await reg.pushManager.getSubscription();
      if (existingSub) {
        await existingSub.unsubscribe();
      }

      // 4. Try to subscribe
      // Ensure VAPID_PUBLIC_KEY is a clean string without whitespace
      const convertedKey = urlBase64ToUint8Array(VAPID_PUBLIC_KEY.trim());

      const subscription = await reg.pushManager.subscribe({
        userVisibleOnly: true,
        applicationServerKey: convertedKey,
      });

      // 5. Sync with your backend
      await postSubscription(subscription);

      localStorage.setItem("notifications_allowed", "true");
      notificationPermission = "granted";
      notificationSubscribed = true;

      console.log("Subscription successful!");
    } catch (e: any) {
      console.error("Full Subscription Error:", e);
      notificationError =
        e.name === "AbortError"
          ? "Subscription timed out or was blocked by Chrome."
          : e.message;
      notificationPermission = Notification.permission as PermissionState;
    } finally {
      notificationLoading = false;
    }
  }

  $effect(() => {
    if (!auth.ready) return;

    fetch_sessions();
  });

  onMount(async () => {
    if (
      typeof Notification === "undefined" ||
      !("serviceWorker" in navigator)
    ) {
      notificationPermission = "unsupported";
      return;
    }

    notificationPermission = Notification.permission as PermissionState;

    // check if already subscribed at browser level
    const reg = await navigator.serviceWorker.ready;
    const existing = await reg.pushManager.getSubscription();

    if (existing && Notification.permission === "granted") {
      // re-POST to backend in case localStorage was cleared
      if (localStorage.getItem("notifications_allowed") !== "true") {
        await postSubscription(existing);
      }
      notificationSubscribed = true;
      localStorage.setItem("notifications_allowed", "true");
    }
  });
</script>

<svelte:head>
  <title>DevinLittle.net - Settings</title>
  <meta name="description" content="The settings page of DevinLittle.net" />
</svelte:head>

{#if auth.ready}
  <div class="wrap">
    <h1>Settings</h1>

    {#if notificationPermission === "unsupported"}
      <div class="card">
        <p class="card-title">Notifications</p>
        <p class="card-sub">
          Receieve Notifications even when off the website!
        </p>
        <br />
        <p>Your browser does not support push notifications.</p>
      </div>
    {:else if notificationPermission === "denied"}
      <div class="card">
        <p class="card-title">Notifications</p>
        <p class="card-sub">
          Receieve Notifications even when off the website!
        </p>
        <br />
        <p>
          Notifications are blocked. Please enable them in your browser
          settings.
        </p>
      </div>
    {:else if notificationSubscribed}
      <div class="card">
        <p class="card-title">Notifications</p>
        <p class="card-sub">
          Receieve Notifications even when off the website!
        </p>
        <br />
        <p>Notifications are enabled.</p>

        <!-- <button
          onclick={() => {
            localStorage.removeItem("notifications_allowed");
          }}>Reset notifications</button
        > !-->
      </div>
    {:else}
      <div class="card">
        <p class="card-title">Notifications</p>
        <p class="card-sub">
          Receieve Notifications even when off the website!
        </p>
        <br />
        <button onclick={enableNotifications} disabled={notificationLoading}>
          {notificationLoading ? "Enabling..." : "Enable notifications"}
        </button>
        {#if notificationError}
          <p>{notificationError}</p>
        {/if}
      </div>
    {/if}

    <!-- ENCRYPTED MESSAGING CARD -->
    {#if ["trusted", "devin", "owen"].includes(getRole(auth.roles, "smalltalk"))}
      <div class="card">
        <p class="card-title">Encrypted Messaging</p>
        <p class="card-sub">
          Manage your SmallTalk encryption key and recovery options.
        </p>

        {#if db_state.needs_onboarding}
          <div class="info-banner">
            <svg
              width="16"
              height="16"
              viewBox="0 0 24 24"
              fill="none"
              stroke="currentColor"
              stroke-width="2"
            >
              <circle cx="12" cy="12" r="10" /><path d="M12 8v4M12 16h.01" />
            </svg>
            SmallTalk is not set up on this device yet.
          </div>
          <button class="btn btn-primary" onclick={startOnboarding}>
            Set up encrypted messaging
          </button>
        {:else if db_state.needs_key_sync}
          <div class="info-banner warn">
            <svg
              width="16"
              height="16"
              viewBox="0 0 24 24"
              fill="none"
              stroke="currentColor"
              stroke-width="2"
            >
              <path
                d="M10.29 3.86L1.82 18a2 2 0 0 0 1.71 3h16.94a2 2 0 0 0 1.71-3L13.71 3.86a2 2 0 0 0-3.42 0z"
              /><path d="M12 9v4M12 17h.01" />
            </svg>
            No encryption key found on this device.
          </div>

          <!-- emoji handshake section -->
          <div class="enc-section">
            <p class="enc-label">Transfer from another device</p>
            <p class="enc-hint">
              Make sure another device with your key is online and logged in.
            </p>

            {#if keysync.status === "idle"}
              <button class="btn" onclick={requestKeySync}>
                Transfer key from online device
              </button>
            {:else if keysync.status === "requesting"}
              <div class="emoji-status">
                <span class="spinner"></span>
                Waiting for another device to respond...
              </div>
              <button class="btn btn-ghost" onclick={cancelKeySync}
                >Cancel</button
              >
            {:else if keysync.status === "pending_click"}
              <p class="emoji-instruction">
                Look at your other device and click the highlighted emoji:
              </p>
              <div class="emoji-grid">
                {#each keysync.challenge_emojis as emoji}
                  <button
                    class="emoji-btn"
                    onclick={() => respondToChallenge(emoji)}
                  >
                    {emoji}
                  </button>
                {/each}
              </div>
              <button class="btn btn-ghost" onclick={cancelKeySync}
                >Cancel</button
              >
            {:else if keysync.status === "transferring"}
              <div class="emoji-status">
                <span class="spinner"></span>
                Receiving key...
              </div>
            {:else if keysync.status === "complete"}
              <div class="success-banner">
                ✓ Key transferred successfully! Reload to continue.
              </div>
            {:else if keysync.status === "failed"}
              <div class="error-banner">Incorrect emoji. Try again.</div>
              <button class="btn" onclick={requestKeySync}>Try again</button>
            {:else if keysync.status === "expired"}
              <div class="error-banner">Challenge expired. Try again.</div>
              <button class="btn" onclick={requestKeySync}>Try again</button>
            {/if}
          </div>

          <div class="divider-row"><span>or</span></div>

          <!-- recovery phrase section -->
          <div class="enc-section">
            <p class="enc-label">Recover with phrase</p>
            <p class="enc-hint">
              Enter your 12 word recovery phrase to restore your key.
            </p>

            {#if !show_recovery_recover}
              <button
                class="btn"
                onclick={() => (show_recovery_recover = true)}
              >
                Recover with 12 word phrase
              </button>
            {:else}
              <div class="word-grid">
                {#each recovery_input as _, i}
                  <div class="word-input-wrap">
                    <span class="word-num">{i + 1}</span>
                    <input
                      type="text"
                      bind:value={recovery_input[i]}
                      placeholder="word"
                      class="word-input"
                    />
                  </div>
                {/each}
              </div>
              {#if recovery_error}
                <div class="error-banner">{recovery_error}</div>
              {/if}
              <div class="btn-row">
                <button
                  class="btn btn-ghost"
                  onclick={() => {
                    show_recovery_recover = false;
                    recovery_error = "";
                  }}
                >
                  Cancel
                </button>
                <button
                  class="btn btn-primary"
                  onclick={attemptRecovery}
                  disabled={recovery_loading}
                >
                  {recovery_loading ? "recovering..." : "Recover key"}
                </button>
              </div>
            {/if}
          </div>
        {:else if db_state.ready}
          <!-- key is ready, show management options -->
          <div class="enc-status">
            <svg
              width="14"
              height="14"
              viewBox="0 0 24 24"
              fill="none"
              stroke="#5dcaa5"
              stroke-width="2.5"
            >
              <path d="M20 6L9 17l-5-5" />
            </svg>
            Encryption key active on this device
          </div>

          <!-- trusted device: show emoji challenge if another device requested -->
          {#if keysync.status === "challenging"}
            <div class="enc-section highlight">
              <p class="enc-label">Another device wants your key</p>
              <p class="emoji-instruction">
                The highlighted emoji will appear on your other device. It is:
              </p>
              <div class="emoji-grid">
                {#each keysync.challenge_emojis as emoji}
                  <div
                    class="emoji-display"
                    class:correct={emoji === keysync.correct_emoji}
                  >
                    {emoji}
                  </div>
                {/each}
              </div>
              <p class="enc-hint">
                Waiting for the other device to click the correct emoji...
              </p>
              <button class="btn btn-ghost" onclick={cancelKeySync}
                >Cancel</button
              >
            </div>
          {/if}

          {#if keysync.status === "complete"}
            <div class="success-banner">
              ✓ Key transferred to other device successfully!
            </div>
          {/if}

          <div class="enc-section">
            <p class="enc-label">Recovery phrase</p>
            <p class="enc-hint">
              Set up a 12 word recovery phrase in case you lose access to all
              your devices.
            </p>

            {#if !show_recovery_setup}
              <button class="btn" onclick={startRecoverySetup}>
                Set up recovery phrase
              </button>
            {:else}
              <div class="warn">
                Write these words down and store them safely. We will never show
                them again.
              </div>
              <div class="word-grid display">
                {#each recovery_words as word, i}
                  <div class="word-display">
                    <span class="word-num">{i + 1}</span>
                    <span class="word-val">{word}</span>
                  </div>
                {/each}
              </div>
              <label class="checkbox-row">
                <input type="checkbox" bind:checked={recovery_confirmed} />
                I've written these down and stored them safely
              </label>
              <div class="btn-row">
                <button
                  class="btn btn-ghost"
                  onclick={() => {
                    show_recovery_setup = false;
                    recovery_words = [];
                  }}
                >
                  Cancel
                </button>
                <button
                  class="btn btn-primary"
                  onclick={confirmRecoverySetup}
                  disabled={!recovery_confirmed || recovery_loading}
                >
                  {recovery_loading ? "saving..." : "Save recovery phrase"}
                </button>
              </div>
            {/if}
          </div>
        {/if}
      </div>
    {/if}

    <!-- SCHOOLOGY/GRADEGETTER CARD -->

    <!--    {#if ["trusted", "devin", "owen"].includes(getRole(auth.roles, "gradegetter"))} -->
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
    <!-- {/if} -->

    <!-- ACTIVE SESSIONS CARD -->
    <div class="card">
      <p class="card-title">Active sessions</p>
      <p class="card-sub">All devices currently signed in to your account.</p>

      {#each sessions as session}
        {@const { browser, os, icon } = parseUA(session.user_agent)}
        <div class="session-row">
          <div class="session-icon">
            {#if icon === "mobile"}
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
          <button
            class="btn btn-danger"
            onclick={() => revoke(session.session_id)}>Revoke</button
          >
        </div>
      {/each}

      <button class="btn btn-danger btn-full" onclick={revokeAll}>
        Sign out of all sessions
      </button>
    </div>

    <!-- DELETE ACCOUNT CARD -->
    <div class="card">
      <p class="card-title">Delete account</p>
      <p class="card-sub">Permanently remove your account and all data.</p>
      <div class="warn">This cannot be undone.</div>

      {#if !showConfirm}
        <button class="btn btn-danger" onclick={() => (showConfirm = true)}>
          Delete my account
        </button>
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
  <h1 style="text-align:center; margin-top: 4rem;">Login to View Settings</h1>
{/if}

<style>
  .wrap {
    max-width: var(--column-width);
    margin: var(--column-margin-top) auto;
    padding: 0 1rem 4rem;
    display: flex;
    flex-direction: column;
    gap: 1.5rem;
  }

  h1 {
    font-size: 2.5rem;
    font-weight: 600;
    color: var(--color-theme-1);
    margin: 0;
  }

  .card {
    background: rgba(255, 255, 255, 0.02);
    border: 1px solid var(--color-border);
    border-radius: 1rem;
    padding: 1.5rem;
    backdrop-filter: blur(6px);
    box-shadow: 0 4px 12px rgba(0, 0, 0, 0.3);
    display: flex;
    flex-direction: column;
    gap: 0.75rem;
  }

  .card-title {
    font-size: 1.1rem;
    font-weight: 600;
    color: var(--color-theme-2);
    border-bottom: 1px solid var(--color-border);
    padding-bottom: 0.6rem;
    margin: 0;
  }

  .card-sub {
    font-size: 0.85rem;
    color: var(--color-subtle-text);
    margin: 0;
  }

  /* banners */
  .info-banner {
    display: flex;
    align-items: center;
    gap: 0.5rem;
    background: rgba(126, 156, 255, 0.08);
    border: 1px solid rgba(126, 156, 255, 0.2);
    border-radius: 0.5rem;
    padding: 0.65rem 0.85rem;
    font-size: 0.85rem;
    color: var(--color-theme-1);
  }

  .info-banner.warn {
    background: rgba(239, 159, 39, 0.08);
    border-color: rgba(239, 159, 39, 0.2);
    color: #ef9f27;
  }

  .success-banner {
    background: rgba(93, 202, 165, 0.1);
    border: 1px solid rgba(93, 202, 165, 0.25);
    border-radius: 0.5rem;
    padding: 0.65rem 0.85rem;
    font-size: 0.85rem;
    color: #5dcaa5;
  }

  .error-banner {
    background: rgba(255, 100, 100, 0.08);
    border: 1px solid rgba(255, 100, 100, 0.2);
    border-radius: 0.5rem;
    padding: 0.65rem 0.85rem;
    font-size: 0.85rem;
    color: #ff9090;
  }

  .enc-status {
    display: flex;
    align-items: center;
    gap: 0.4rem;
    font-size: 0.85rem;
    color: #5dcaa5;
  }

  /* enc sections */
  .enc-section {
    background: rgba(255, 255, 255, 0.02);
    border: 1px solid var(--color-border);
    border-radius: 0.75rem;
    padding: 1rem;
    display: flex;
    flex-direction: column;
    gap: 0.6rem;
  }

  .enc-section.highlight {
    border-color: rgba(126, 156, 255, 0.3);
    background: rgba(126, 156, 255, 0.04);
  }

  .enc-label {
    font-size: 0.85rem;
    font-weight: 600;
    color: var(--color-text);
    margin: 0;
  }

  .enc-hint {
    font-size: 0.78rem;
    color: var(--color-subtle-text);
    margin: 0;
    line-height: 1.5;
  }

  .divider-row {
    display: flex;
    align-items: center;
    gap: 0.75rem;
    color: var(--color-subtle-text);
    font-size: 0.8rem;
  }

  .divider-row::before,
  .divider-row::after {
    content: "";
    flex: 1;
    height: 1px;
    background: var(--color-border);
  }

  /* emoji ui */
  .emoji-status {
    display: flex;
    align-items: center;
    gap: 0.5rem;
    font-size: 0.85rem;
    color: var(--color-subtle-text);
  }

  .emoji-instruction {
    font-size: 0.85rem;
    color: var(--color-text);
    margin: 0;
  }

  .emoji-grid {
    display: flex;
    gap: 0.5rem;
    flex-wrap: wrap;
  }

  .emoji-btn {
    font-size: 1.75rem;
    background: rgba(255, 255, 255, 0.04);
    border: 1px solid var(--color-border);
    border-radius: 0.5rem;
    padding: 0.5rem 0.75rem;
    cursor: pointer;
    transition: all 0.15s ease;
    line-height: 1;
  }

  .emoji-btn:hover {
    background: rgba(126, 156, 255, 0.12);
    border-color: rgba(126, 156, 255, 0.4);
    transform: scale(1.1);
  }

  .emoji-display {
    font-size: 1.75rem;
    background: rgba(255, 255, 255, 0.04);
    border: 1px solid var(--color-border);
    border-radius: 0.5rem;
    padding: 0.5rem 0.75rem;
    line-height: 1;
    transition: all 0.15s ease;
  }

  .emoji-display.correct {
    background: rgba(126, 156, 255, 0.18);
    border-color: rgba(126, 156, 255, 0.5);
    box-shadow: 0 0 12px rgba(126, 156, 255, 0.3);
    transform: scale(1.12);
  }

  /* word grid */
  .word-grid {
    display: grid;
    grid-template-columns: repeat(3, 1fr);
    gap: 0.4rem;
  }

  .word-input-wrap {
    display: flex;
    align-items: center;
    gap: 0.35rem;
    background: rgba(255, 255, 255, 0.03);
    border: 1px solid var(--color-border);
    border-radius: 0.4rem;
    padding: 0.3rem 0.5rem;
  }

  .word-num {
    font-size: 0.7rem;
    color: var(--color-subtle-text);
    font-family: var(--font-mono);
    min-width: 1rem;
    flex-shrink: 0;
  }

  .word-input {
    background: none;
    border: none;
    color: var(--color-text);
    font-size: 0.82rem;
    font-family: var(--font-mono);
    width: 100%;
    padding: 0;
    outline: none;
  }

  .word-grid.display .word-display {
    display: flex;
    align-items: center;
    gap: 0.35rem;
    background: rgba(255, 255, 255, 0.03);
    border: 1px solid var(--color-border);
    border-radius: 0.4rem;
    padding: 0.4rem 0.6rem;
  }

  .word-val {
    font-size: 0.85rem;
    font-family: var(--font-mono);
    color: var(--color-theme-1);
  }

  .checkbox-row {
    display: flex;
    align-items: center;
    gap: 0.5rem;
    font-size: 0.82rem;
    color: var(--color-subtle-text);
    cursor: pointer;
  }

  /* sessions */
  .session-row {
    display: flex;
    align-items: center;
    gap: 0.75rem;
    background: rgba(255, 255, 255, 0.04);
    padding: 0.75rem 1rem;
    border-radius: 0.5rem;
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

  /* buttons */
  .btn {
    display: inline-flex;
    align-items: center;
    justify-content: center;
    gap: 0.4rem;
    padding: 0.5rem 1rem;
    border-radius: 0.4rem;
    font-size: 0.875rem;
    font-family: var(--font-body);
    cursor: pointer;
    transition: all 0.15s ease;
    border: 1px solid var(--color-border);
    background: rgba(255, 255, 255, 0.04);
    color: var(--color-text);
    align-self: flex-start;
  }

  .btn:hover {
    background: rgba(255, 255, 255, 0.09);
  }
  .btn:disabled {
    opacity: 0.5;
    cursor: not-allowed;
  }

  .btn-primary {
    background: rgba(126, 156, 255, 0.15);
    border-color: rgba(126, 156, 255, 0.4);
    color: var(--color-theme-1);
  }

  .btn-primary:hover:not(:disabled) {
    background: rgba(126, 156, 255, 0.25);
  }

  .btn-ghost {
    background: transparent;
    color: var(--color-subtle-text);
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
  }

  .btn-row {
    display: flex;
    gap: 0.5rem;
    align-items: center;
  }

  /* misc */
  .warn {
    background: rgba(255, 100, 100, 0.07);
    border: 1px solid rgba(255, 100, 100, 0.2);
    border-radius: 0.5rem;
    padding: 0.75rem 1rem;
    font-size: 0.85rem;
    color: #ff9090;
  }

  .confirm-label {
    font-size: 0.85rem;
    color: var(--color-subtle-text);
    margin: 0;
  }

  .confirm-row {
    display: flex;
    gap: 0.5rem;
  }

  .spinner {
    width: 14px;
    height: 14px;
    border: 2px solid rgba(126, 156, 255, 0.2);
    border-top-color: var(--color-theme-1);
    border-radius: 50%;
    animation: spin 0.8s linear infinite;
    flex-shrink: 0;
  }

  @keyframes spin {
    to {
      transform: rotate(360deg);
    }
  }

  @media (max-width: 600px) {
    .word-grid {
      grid-template-columns: repeat(2, 1fr);
    }
    .emoji-grid {
      gap: 0.35rem;
    }
    .btn-row {
      flex-direction: column;
    }
    .btn-row .btn {
      width: 100%;
    }
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
</style>
