<script>
  import Header from "./Header.svelte";

  import { onMount } from "svelte";
  import {
    auth,
    get_ready_for_devin_grfd,
    initAuth,
  } from "$lib/utils/auth.svelte";
  import {
    connectNotifications,
    socketState,
  } from "$lib/utils/notifications.svelte";

  onMount(async () => {
    await get_ready_for_devin_grfd(false);

    document.addEventListener("visibilitychange", async () => {
      if (document.visibilityState === "visible" && getSocket() === null) {
        await initAuth();
        connectNotifications();
      }
    });
  });

  $effect(async () => {
    if (socketState.value == "disconnected") {
      console.log("connecting as was previously disconnected");
      await initAuth();
      connectNotifications();
    }
  });

  $inspect(auth);
  $inspect(nanopass);
  $inspect(smalltalk_notes);

  import "./styles.css";
  import Notifications from "$lib/comps/Notifications.svelte";
  import { nanopass } from "$lib/utils/nanopass.svelte";
  import { smalltalk_notes } from "$lib/utils/smalltalk.svelte";
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
