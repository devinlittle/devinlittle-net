<script>
  import {
    notifications,
    registerDismiss,
    removeNotification,
  } from "$lib/utils/notifications.svelte.js";
  import { onMount } from "svelte";

  let dying = $state(new Set());

  function dismiss(id) {
    dying = new Set([...dying, id]);
    setTimeout(() => {
      dying.delete(id);
      dying = new Set(dying);
      removeNotification(id);
    }, 500);
  }

  function exitStyle(id) {
    const seed = id.charCodeAt(0) + id.charCodeAt(4);
    const x = ((seed * 137) % 800) - 400;
    const y = ((seed * 53) % 600) - 300;
    const rot = ((seed * 31) % 720) - 360;
    return `--ex: ${x}px; --ey: ${y}px; --erot: ${rot}deg;`;
  }

  onMount(() => {
    registerDismiss(dismiss);
  });
</script>

<div class="notif-container">
  {#each notifications as n (n.id)}
    <div
      class="notif"
      class:global={n.global}
      class:offer={n.type === "offer"}
      class:dying={dying.has(n.id)}
      style={exitStyle(n.id)}
    >
      <div class="notif-header">
        <span class="notif-title">{n.title}</span>
        {#if n.sender || n.global}
          <span class="notif-sender">{n.global ? "global" : n.sender}</span>
        {/if}
        <button class="notif-close" onclick={() => dismiss(n.id)}>✕</button>
      </div>
      <p class="notif-body">{n.body}</p>
      {#if n.type === "offer"}
        <div class="notif-actions">
          <button
            class="accept"
            onclick={() => {
              // TODO: webrtc answer
              dismiss(n.id);
            }}>Accept</button
          >
          <button class="deny" onclick={() => dismiss(n.id)}>Deny</button>
        </div>
      {/if}
    </div>
  {/each}
</div>

<style>
  .notif-container {
    position: fixed;
    top: 1rem;
    right: 1rem;
    z-index: 9999;
    display: flex;
    flex-direction: column;
    gap: 0.5rem;
    align-items: flex-end;
  }

  .notif {
    background: rgba(20, 10, 35, 0.97);
    border: 1px solid var(--color-border);
    border-radius: 0.6rem;
    padding: 0.75rem 1rem;
    width: 18rem;
    box-shadow: 0 8px 32px rgba(0, 0, 0, 0.7);
    backdrop-filter: blur(12px);
    animation: slide-in 0.35s cubic-bezier(0.175, 0.885, 0.32, 1.6);
    border-left: 3px solid var(--color-theme-1);
  }

  .notif.global {
    border-left-color: #ff4444;
  }

  .notif.offer {
    border-left-color: var(--color-theme-2);
  }

  .notif.dying {
    animation: spin-away 0.5s cubic-bezier(0.55, 0, 1, 0.45) forwards;
    pointer-events: none;
  }

  .notif-header {
    display: flex;
    align-items: center;
    gap: 0.4rem;
    margin-bottom: 0.35rem;
  }

  .notif-title {
    font-weight: 600;
    font-size: 0.8rem;
    color: var(--color-theme-2);
    flex: 1;
  }

  .notif.global .notif-title {
    color: #ff6666;
  }

  .notif-sender {
    font-size: 0.72rem;
    color: var(--color-subtle-text);
    font-family: var(--font-mono);
  }

  .notif-close {
    background: none;
    border: none;
    color: var(--color-subtle-text);
    cursor: pointer;
    padding: 0 0.2rem;
    font-size: 0.75rem;
    line-height: 1;
    transition: color 0.15s ease;
  }

  .notif-close:hover {
    color: #ff9090;
    background: none;
  }

  .notif-body {
    font-size: 0.82rem;
    color: var(--color-subtle-text);
    margin: 0;
    line-height: 1.5;
  }

  .notif-actions {
    display: flex;
    gap: 0.4rem;
    margin-top: 0.6rem;
  }

  .notif-actions button {
    flex: 1;
    padding: 0.35rem 0.5rem;
    font-size: 0.78rem;
    border-radius: 0.35rem;
    transition: background 0.15s ease;
  }

  .accept {
    background: rgba(126, 156, 255, 0.12);
    border: 1px solid rgba(126, 156, 255, 0.4);
    color: var(--color-theme-1);
    font-weight: 600;
  }

  .accept:hover {
    background: rgba(126, 156, 255, 0.25);
  }

  .deny {
    background: rgba(255, 68, 68, 0.08);
    border: 1px solid rgba(255, 68, 68, 0.3);
    color: #ff9090;
  }

  .deny:hover {
    background: rgba(255, 68, 68, 0.18);
  }

  @keyframes slide-in {
    from {
      opacity: 0;
      transform: translateX(110%);
    }
    to {
      opacity: 1;
      transform: translateX(0);
    }
  }

  @keyframes spin-away {
    0% {
      opacity: 1;
      transform: translate(0, 0) rotate(0deg) scale(1);
    }
    100% {
      opacity: 0;
      transform: translate(var(--ex), var(--ey)) rotate(var(--erot)) scale(0.1);
    }
  }
</style>
