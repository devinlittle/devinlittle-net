<script>
  import { page } from "$app/state";
  import { auth, logout } from "$lib/utils/auth.svelte.js";
  import github from "$lib/images/github.svg";
  import { goto } from "$app/navigation";

  let open = $state(false);
  function toggle() {
    open = !open;
  }
  function close() {
    open = false;
  }
</script>

<header>
  <nav>
    <ul>
      <li aria-current={page.url.pathname === "/" ? "page" : undefined}>
        <a href="/">Home</a>
      </li>
      <li aria-current={page.url.pathname === "/projects" ? "page" : undefined}>
        <a href="/projects">Projects</a>
      </li>
      <li
        aria-current={page.url.pathname === "/gradegetter" ? "page" : undefined}
      >
        <a href="/gradegetter">GradeGetter</a>
      </li>
    </ul>

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
  </nav>

  <div class="corner">
    <a href="https://github.com/devinlittle/devinlittlenet">
      <img src={github} alt="GitHub" />
    </a>
  </div>
</header>

<style>
  header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    padding: 0 0.5rem;
    background: rgba(41, 11, 68, 0.7);
    backdrop-filter: blur(6px);
    border-bottom: 1px solid var(--color-border);
    /*    position: sticky; */
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
  }

  ul {
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
</style>
