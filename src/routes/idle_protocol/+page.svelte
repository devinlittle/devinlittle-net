<script>
  import { onMount } from "svelte";
  import init from "$lib/idle_protocol/idle_protocol";
  let clicked = $state(false);

  async function load() {
    clicked = true;
    await init();
  }
</script>

<svelte:head>
  <title>DevinLittle.net - Idle Protocol</title>
</svelte:head>

{#if clicked}
  <div class="game-container">
    <canvas id="bevy"></canvas>
  </div>
{:else}
  <button onclick={load}>Load!</button>
{/if}

<style>
  .game-container {
    width: 1280px; /* Set desired width */
    height: 800px; /* Set desired height */
    overflow: hidden; /* Prevent overflow */
  }
</style>
