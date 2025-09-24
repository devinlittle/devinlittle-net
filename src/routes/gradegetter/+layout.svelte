<script>
  import { page } from "$app/state";
  import { onMount } from "svelte";
  let { children } = $props();

  let LoggedIn = $state(false);

  let load = async () => {
    if (localStorage.getItem("token") === null) {
      LoggedIn = false;
    } else {
      LoggedIn = true;
    }
  };

  onMount(() => {
    load();
  });
</script>

<header>
  <div class="ggHeaderDiv">
    <nav>
      <ul>
        <li
          aria-current={page.url.pathname === "/gradegetter"
            ? "page"
            : undefined}
        >
          <a href="/gradegetter">Grades</a>
        </li>
        {#if !LoggedIn}
          <li
            aria-current={page.url.pathname === "/gradegetter/login"
              ? "page"
              : undefined}
          >
            <a href="/gradegetter/login">Login</a>
          </li>
          <li
            aria-current={page.url.pathname === "/gradegetter/register"
              ? "page"
              : undefined}
          >
            <a href="/gradegetter/register">Register</a>
          </li>
        {/if}
      </ul>
    </nav>
  </div>
</header>

<main>
  {@render children?.()}
</main>

<style>
  header {
    background-color: rgba(255, 255, 255, 0.02);
    border-bottom: 1px solid var(--color-border);
    box-shadow: 0 4px 12px rgba(0, 0, 0, 0.25);
    backdrop-filter: blur(6px);
    padding: 0.75rem 1rem;
    position: sticky;
    top: 0;
    z-index: 100;
  }

  nav {
    max-width: var(--column-width);
    margin: 0 auto;
  }

  nav ul {
    display: flex;
    justify-content: center;
    align-items: center;
    gap: 1.5rem;
    list-style: none;
    padding: 0;
    margin: 0;
  }

  nav li {
    position: relative;
  }

  nav a {
    color: var(--color-text);
    text-decoration: none;
    text-transform: uppercase;
    font-size: 0.85rem;
    font-weight: 600;
    letter-spacing: 0.08em;
    padding: 0.5rem 0;
    display: inline-block;
    transition: color 0.2s ease;
    border-bottom: 2px solid transparent;
  }

  nav a:hover {
    color: var(--color-theme-2);
  }

  li[aria-current="page"] a {
    color: var(--color-theme-1);
    border-bottom: 2px solid var(--color-theme-1);
  }

  /*   header {
    display: flex;
    justify-content: space-between;
  }

  .ggHeaderDiv {
    background-color: var(--color-bg-2);
  }

  ul {
    position: relative;
    padding: 0;
    margin: 0;
    height: 3em;
    display: flex;
    justify-content: center;
    align-items: center;
    list-style: none;
    background: var(--background);
    background-size: contain;
  }

  li {
    position: relative;
    height: 100%;
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
  } */
</style>
