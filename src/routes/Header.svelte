<script lang="ts">
  import { page } from "$app/state";
  import { auth, logout } from "$lib/utils/auth.svelte.js";
  import github from "$lib/images/github.svg";
  import { goto } from "$app/navigation";

  let open = $state(false);
  let mobileOpen = $state(false);

  function toggle() {
    open = !open;
  }
  function close() {
    open = false;
  }
  function toggleMobile() {
    mobileOpen = !mobileOpen;
  }
  function closeMobile() {
    mobileOpen = false;
  }

  const links = [
    { href: "/", label: "Home" },
    { href: "/projects", label: "Projects" },
    { href: "/gradegetter", label: "GradeGetter" },
    { href: "/nanopass", label: "NanoPass" },
    //    { href: "/smalltalk", label: "SmallTalk" },
    { href: "/calc", label: "Calculator" },
  ];
</script>

<header>
  <nav>
    <!-- desktop links -->
    <ul class="desktop-links">
      {#each links as link}
        <li aria-current={page.url.pathname === link.href ? "page" : undefined}>
          <a href={link.href} onclick={closeMobile}>{link.label}</a>
        </li>
      {/each}
    </ul>

    <!-- desktop user menu -->
    <div class="desktop-auth">
      {#if !auth.ready || !auth.username}
        <a href="/auth" class="auth-link">Login / Sign up</a>
      {:else}
        <div class="user-menu">
          <button class="user-btn" onclick={toggle}>
            <svg
              xmlns="http://www.w3.org/2000/svg"
              width="15"
              height="15"
              viewBox="0 0 24 24"
              fill="none"
              stroke="currentColor"
              stroke-width="2"
            >
              <path d="M20 21v-2a4 4 0 0 0-4-4H8a4 4 0 0 0-4 4v2" />
              <circle cx="12" cy="7" r="4" />
            </svg>
            {auth.username}
            <svg
              xmlns="http://www.w3.org/2000/svg"
              width="11"
              height="11"
              viewBox="0 0 24 24"
              fill="none"
              stroke="currentColor"
              stroke-width="2.5"
              style="opacity:0.6"
            >
              <polyline points="6 9 12 15 18 9" />
            </svg>
          </button>
          {#if open}
            <div class="dropdown">
              <button
                onclick={() => {
                  goto("/settings");
                  close();
                }}>Settings</button
              >
              <hr />
              <button
                class="danger"
                onclick={() => {
                  logout();
                  close();
                }}>Log out</button
              >
            </div>
          {/if}
        </div>
      {/if}
    </div>

    <!-- mobile hamburger -->
    <button class="hamburger" onclick={toggleMobile} aria-label="menu">
      {#if mobileOpen}
        <svg
          width="20"
          height="20"
          viewBox="0 0 24 24"
          fill="none"
          stroke="currentColor"
          stroke-width="2.5"
        >
          <path d="M18 6L6 18M6 6l12 12" />
        </svg>
      {:else}
        <svg
          width="20"
          height="20"
          viewBox="0 0 24 24"
          fill="none"
          stroke="currentColor"
          stroke-width="2.5"
        >
          <path d="M3 6h18M3 12h18M3 18h18" />
        </svg>
      {/if}
    </button>
  </nav>

  <div class="corner">
    <a href="https://github.com/devinlittle/devinlittlenet">
      <img src={github} alt="GitHub" />
    </a>
  </div>
</header>

<!-- mobile drawer -->
{#if mobileOpen}
  <div class="mobile-drawer">
    <ul>
      {#each links as link}
        <li aria-current={page.url.pathname === link.href ? "page" : undefined}>
          <a href={link.href} onclick={closeMobile}>{link.label}</a>
        </li>
      {/each}
    </ul>
    <div class="mobile-auth">
      {#if !auth.ready || !auth.username}
        <a href="/auth" class="auth-link" onclick={closeMobile}
          >Login / Sign up</a
        >
      {:else}
        <div class="mobile-user">
          <svg
            xmlns="http://www.w3.org/2000/svg"
            width="15"
            height="15"
            viewBox="0 0 24 24"
            fill="none"
            stroke="currentColor"
            stroke-width="2"
          >
            <path d="M20 21v-2a4 4 0 0 0-4-4H8a4 4 0 0 0-4 4v2" />
            <circle cx="12" cy="7" r="4" />
          </svg>
          <span>{auth.username}</span>
        </div>
        <button
          onclick={() => {
            goto("/settings");
            closeMobile();
          }}>Settings</button
        >
        <button
          class="danger"
          onclick={() => {
            logout();
            closeMobile();
          }}>Log out</button
        >
      {/if}
    </div>
  </div>
{/if}

<style>
  header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    padding: 0 0.5rem;
    background: rgba(41, 11, 68, 0.7);
    backdrop-filter: blur(6px);
    border-bottom: 1px solid var(--color-border);
    top: 0;
    z-index: 50;
  }

  .corner {
    width: 3em;
    height: 3em;
    flex-shrink: 0;
  }
  .corner a {
    display: flex;
    align-items: center;
    justify-content: center;
    width: 100%;
    height: 100%;
  }
  .corner img {
    width: 2em;
    height: 2em;
    object-fit: contain;
    opacity: 0.8;
    transition: opacity 0.2s ease;
  }
  .corner img:hover {
    opacity: 1;
  }

  nav {
    display: flex;
    align-items: center;
    gap: 1rem;
    flex: 1;
  }

  .desktop-links {
    padding: 0;
    margin: 0;
    height: 3em;
    display: flex;
    align-items: center;
    list-style: none;
    gap: 0.25rem;
  }

  li {
    position: relative;
    height: 100%;
    display: flex;
    align-items: center;
  }

  li[aria-current="page"]::before {
    --size: 6px;
    content: "";
    width: 0;
    height: 0;
    position: absolute;
    top: 0;
    left: calc(50% - var(--size));
    border: var(--size) solid transparent;
    border-top: var(--size) solid var(--color-theme-1);
  }

  nav a {
    display: flex;
    height: 100%;
    align-items: center;
    padding: 0 0.5rem;
    color: var(--color-text);
    font-weight: 700;
    font-size: 0.8rem;
    text-transform: uppercase;
    letter-spacing: 0.1em;
    text-decoration: none;
    transition: color 0.2s linear;
  }
  nav a:hover {
    color: var(--color-theme-1);
  }

  .auth-link {
    font-size: 0.8rem;
    font-weight: 700;
    letter-spacing: 0.05em;
    text-transform: uppercase;
    padding: 0.4rem 0.75rem;
    border: 1px solid var(--color-border);
    border-radius: 0.5rem;
    background: rgba(255, 255, 255, 0.04);
    transition:
      background 0.2s ease,
      color 0.2s ease;
    height: auto;
  }
  .auth-link:hover {
    background: rgba(255, 255, 255, 0.1);
    color: var(--color-theme-1);
  }

  .user-menu {
    position: relative;
  }
  .user-btn {
    display: flex;
    align-items: center;
    gap: 0.4rem;
    padding: 0.4rem 0.75rem;
    background: rgba(255, 255, 255, 0.04);
    border: 1px solid var(--color-border);
    border-radius: 0.5rem;
    color: var(--color-text);
    font-size: 0.8rem;
    font-weight: 700;
    letter-spacing: 0.05em;
    text-transform: uppercase;
    cursor: pointer;
    transition: background 0.2s ease;
  }
  .user-btn:hover {
    background: rgba(255, 255, 255, 0.1);
  }

  .dropdown {
    position: absolute;
    right: 0;
    top: calc(100% + 0.5rem);
    min-width: 160px;
    background: var(--color-bg-2);
    border: 1px solid var(--color-border);
    border-radius: 0.5rem;
    padding: 0.35rem;
    display: flex;
    flex-direction: column;
    gap: 0.1rem;
    z-index: 100;
    box-shadow: 0 8px 24px rgba(0, 0, 0, 0.4);
    backdrop-filter: blur(6px);
  }
  .dropdown button {
    width: 100%;
    text-align: left;
    padding: 0.5rem 0.75rem;
    background: none;
    border: none;
    border-radius: 0.35rem;
    color: var(--color-text);
    font-size: 0.875rem;
    font-weight: 400;
    letter-spacing: 0;
    text-transform: none;
    cursor: pointer;
    transition: background 0.15s ease;
  }
  .dropdown button:hover {
    background: rgba(255, 255, 255, 0.07);
  }
  .dropdown button.danger {
    color: #ff9090;
  }
  .dropdown button.danger:hover {
    background: rgba(255, 100, 100, 0.1);
  }
  .dropdown hr {
    border: none;
    border-top: 1px solid var(--color-border);
    margin: 0.2rem 0;
  }

  /* hamburger */
  .hamburger {
    display: none;
    background: none;
    border: none;
    color: var(--color-text);
    cursor: pointer;
    padding: 0.4rem;
    /*    margin-left: auto; */
  }

  /* mobile drawer */
  .mobile-drawer {
    display: none;
    flex-direction: column;
    background: rgba(20, 10, 35, 0.98);
    border-bottom: 1px solid var(--color-border);
    backdrop-filter: blur(12px);
    padding: 1rem;
    gap: 0.25rem;
    z-index: 49;
  }

  .mobile-drawer ul {
    list-style: none;
    padding: 0;
    margin: 0;
    display: flex;
    flex-direction: column;
  }

  .mobile-drawer li {
    height: auto;
  }

  .mobile-drawer li[aria-current="page"]::before {
    display: none;
  }

  .mobile-drawer li[aria-current="page"] a {
    color: var(--color-theme-1);
  }

  .mobile-drawer a {
    height: auto;
    padding: 0.75rem 0.5rem;
    font-size: 1rem;
    border-bottom: 1px solid var(--color-border);
    width: 100%;
    color: var(--color-theme-1);
    transition: all 0.2s ease-in-out;
    text-decoration: none;
  }

  .mobile-auth {
    display: flex;
    flex-direction: column;
    gap: 0.5rem;
    padding-top: 1rem;
  }

  .mobile-user {
    display: flex;
    align-items: center;
    gap: 0.5rem;
    color: var(--color-subtle-text);
    font-size: 0.85rem;
    padding: 0.25rem 0.5rem;
  }

  .mobile-auth button {
    text-align: left;
    padding: 0.65rem 0.5rem;
    background: none;
    border: none;
    border-radius: 0.35rem;
    color: var(--color-text);
    font-size: 0.9rem;
    cursor: pointer;
    transition: background 0.15s ease;
    font-family: var(--font-body);
  }

  .mobile-auth button:hover {
    background: rgba(255, 255, 255, 0.07);
  }
  .mobile-auth button.danger {
    color: #ff9090;
  }
  .mobile-auth button.danger:hover {
    background: rgba(255, 100, 100, 0.1);
  }

  @media (max-width: 768px) {
    .desktop-links {
      display: none;
    }
    .desktop-auth {
      display: none;
    }
    .hamburger {
      display: flex;
    }
    .mobile-drawer {
      display: flex;
    }
  }
</style>
