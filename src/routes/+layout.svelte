<script>
  import Header from "./Header.svelte";

  import { onMount } from "svelte";
  import { initAuth, auth } from "$lib/utils/auth.svelte";
  import { db_state, mountDB } from "$lib/utils/sqlite.svelte";
  import {
    addNotification,
    connectNotifications,
    getSocket,
  } from "$lib/utils/notifications.svelte";

  onMount(async () => {
    const ok = await initAuth();
    if (ok) {
      console.log("pretend the db is being setup");
      await mountDB();
      if (db_state.ready) {
      } else if (db_state.needs_key_sync) {
        addNotification({
          type: "important_info",
          title: "Notification",
          body: "messaging capabilities are limited, navigate to the settings page to sync encryption keys",
          sender: "DevinLittle.Net",
          global: false,
        });
      } else if (db_state.needs_onboarding) {
        // generate keys
      }

      connectNotifications();

      document.addEventListener("visibilitychange", async () => {
        if (document.visibilityState === "visible" && getSocket() === null) {
          await initAuth();
          connectNotifications();
        }
      });
    } else {
      // Loading global notifications, no db ever initalized
      connectNotifications();
    }
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
