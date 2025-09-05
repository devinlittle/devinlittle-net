<script>
  import { onMount } from "svelte";

  let loaded = $state(false);

  onMount(() => {
    requestAnimationFrame(() => {
      setTimeout(() => {
        loaded = true;
      }, 0);
    });
  });
</script>

<svelte:head>
  <title>Home</title>
  <meta name="description" content="Devin Little's website yeha" />
</svelte:head>

{#if loaded}
  <div class="intro animate-in">
    <h1>
      <span class="typewriter"
        >Hello, I’m <span class="devin-wrap">Devin </span>
      </span>
    </h1>
    <div class="stats fade-in-delay">
      <span>15 years old</span>
      <span>5+ years learning</span>
      <span>15+ projects</span>
    </div>
  </div>
{/if}

<style>
  .intro {
    height: 75vh;
    color: #7af57f;
    font-family: "Fira Mono", monospace;
    display: flex;
    flex-direction: column;
    justify-content: center;
    align-items: center;
    text-align: center;
    padding: 0 2rem;
    opacity: 0;
    transform: translateY(20px);
    animation: fadeUp 1.2s ease-out forwards;
  }

  h1 {
    font-size: 4rem;
    font-weight: 700;
    margin: 0;
  }

  .devin-wrap {
    display: inline-block;
    position: relative;
    margin-left: 0.5rem;
  }

  .stats {
    margin-top: 2rem;
    font-size: 1.25rem;
    font-weight: 600;
    letter-spacing: 0.1em;
    opacity: 0;
    transform: translateY(10px);
    animation: fadeIn 1.25s ease-out forwards;
    animation-delay: 1.65s;
  }

  .stats span:not(:last-child)::after {
    content: "|";
    margin: 0 1rem;
    color: #4a7a3e;
  }

  /* aimations */
  @keyframes fadeUp {
    to {
      opacity: 1;
      /*      transform: translateY(0); */
    }
  }

  @keyframes fadeIn {
    to {
      opacity: 1;
      transform: translateY(0);
    }
  }

  .typewriter {
    display: inline-block;
    overflow: hidden;
    white-space: nowrap;
    border-right: 2px solid #7af57f;
    animation:
      typing 1.5s steps(25, end),
      blink 0.75s step-end infinite;
  }

  @keyframes typing {
    from {
      width: 0;
    }
    to {
      width: 17ch;
    }
  }

  @keyframes blink {
    50% {
      border-color: transparent;
    }
  }
</style>
