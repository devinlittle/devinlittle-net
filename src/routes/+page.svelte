<script>
  import { onMount } from "svelte";
  import { fly, fade, blur } from "svelte/transition";
  import Page from "./about/+page.svelte";

  let varY = $state();
  let loaded = $state(false);

  onMount(() => {
    requestAnimationFrame(() => {
      setTimeout(() => {
        loaded = true;
      }, 0);
    });
  });

  function onanimationend() {
    anima = true;
  }
  function ontransitionend() {
    anima = true;
  }

  let anima = $state(false);

  let MenuVis = $state(false);

  let MenuVisTog = () => {
    MenuVis = !MenuVis;
  };
  const imgs = ["linux", "proimgs/Sveltekiticon", "rusticon"];
  let selected = $state("1");
  let explainie = $state();
</script>

<svelte:head>
  <title>Home</title>
  <meta name="description" content="Devin Little's website yeha" />
</svelte:head>

<svelte:window bind:scrollY={varY} />

{#if loaded}
  <!-->  <p>{anima}: {selected}</p><!-->
  <h1
    transition:fly={{ x: "100%", duration: 300, delay: 500 }}
    onintroend={onanimationend}
  >
    Hi, I'm Devin!
  </h1>
  {#if anima}
    <div bind:this={explainie} in:fade={{ delay: 500 }} class="Descbox">
      <p
        style="color: pink; display: flex; padding-left: 1rem; text-decoration-style: wavy; justify-content: center;"
      >
        Hover over an icon!
      </p>
      <p style="text-wrap: wrap;">
        I'm learning a lot about all most everything front-end and back-end. I'm
        managing this website through my own servers and I'm not using some UI
        library. Just pure SvelteKit.
      </p>
      <p
        style="color: pink; display: flex; align-self: flex-end; justify-content: center;"
      >
        Hover over an icon!
      </p>
    </div>
    <hr style="padding-top: 1rem; padding-bottom: 1rem; visibility: hidden;" />
    <div in:fly={{ y: "100%", delay: 175, duration: 500 }} class="ImgContainer">
      {#each imgs as item}
        <img
          onclick={MenuVisTog}
          onmouseenter={() => (selected = item)}
          class="card"
          src="{item}.png"
          alt=""
        />
      {/each}
    </div>
    <hr
      style="padding-top: 1rem; padding-bottom: .5rem; hidden; visibility: hidden;"
    />
  {/if}

  <div class="Descbox a">
    {#if selected == "linux"}
      <h1 in:blur={{ delay: 0 }}>Linux</h1>
      <hr class="invi" />
      <p in:fade={{ delay: 0 }}>
        Linux is an os that is free and open source. That means people can read
        and edit the source code. Linux is primarly used in servers like this
        one or in devices like your phone or tv. I am currently running this
        website on a linux server.
      </p>
    {:else if selected == "proimgs/Sveltekiticon"}
      <h1 in:blur={{ delay: 0 }}>Sveltekit</h1>
      <hr class="invi" />
      <p in:fade={{ delay: 0 }}>
        Sveltekit is a meta framework for the Svelte framework. It adds routing
        and a lot more. You can see how I use Sveltekit in the source code
        available in the top right conner. I've chosen Sveltekit because I'm the
        most productive in Svelte rather than over frameworks (Vue, React).
      </p>
    {:else if selected == "rusticon"}
      <h1 in:blur={{ delay: 0 }}>Rust</h1>
      <hr class="invi" />
      <p in:fade={{ delay: 0 }}>
        Rust is a memory safe compiled language that gives you the simplicity of
        Go or Typescript but with the proformance of C++. I use rust to interact
        with my databases using axum, and for cli (Command Line Interface) apps
        with clap.
      </p>
    {/if}
  </div>
{:else}
  <p>Loading...</p>
{/if}

<style>
  img {
    width: 10%;
    height: 10%;
    display: table-cell;
    vertical-align: middle;
    transition: 0.1s;
  }

  .invi {
    color: black;
    border-style: hidden;
    border-spacing: 5rem;
    padding-right: 1rem;
    padding-left: 1rem;
  }

  .Descbox {
    display: flex;
    align-content: center;
    justify-content: center;
    background-color: var(--color-bg-1);
  }

  .Descbox {
    max-height: 350px;
    width: 100%;
    height: 100%;
    box-sizing: border-box;
    vertical-align: middle;
    object-fit: cover;
    position: relative;
    z-index: 1;
    border-radius: 1rem;
    box-shadow: 0 14px 25px rgba(255, 255, 255, 0.16);
  }

  .ImgContainer {
    display: flex;
    align-content: center;
    justify-content: center;
  }

  .card:hover {
    cursor: pointer;
    transform: scale(1.1);
  }
</style>
