<script>
  import Header from "./Header.svelte";

  import { onMount } from "svelte";
  import { initAuth, auth } from "$lib/utils/auth.svelte.js";
  import { connectNotifications } from "$lib/utils/notifications.svelte";

  onMount(async () => {
    await initAuth();
    connectNotifications();
  });

  import "./styles.css";
  import Notifications from "$lib/comps/Notifications.svelte";
  /**
   * @typedef {Object} Props
   * @property {import('svelte').Snippet} [children]
   */

  /** @type {Props} */
  let { children } = $props();
</script>

<div class="app">
  <Header />
  <Notifications />

  <main>
    {@render children()}
  </main>
</div>

<style>
  .app {
    display: flex;
    flex-direction: column;
    min-height: 100vh;
  }

  main {
    flex: 1;
    display: flex;
    flex-direction: column;
    padding: 1rem;
    width: 100%;
    max-width: 64rem;
    margin: 0 auto;
    box-sizing: border-box;
  }
</style>
