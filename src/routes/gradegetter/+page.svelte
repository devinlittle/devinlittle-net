<script lang="ts">
  import { auth } from "$lib/utils/auth.svelte";
  import {
    fetchGrades,
    grades,
    forbidden,
  } from "$lib/utils/gradegetter.svelte";

  $effect(() => {
    if (!auth.ready) return;

    fetchGrades();
  });
</script>

<svelte:head>
  <title>DevinLittle.net - GradeGetter</title>
  <meta name="description" content="Devin's gradegetter duh" />
</svelte:head>

<main>
  {#if auth.ready}
    {#if forbidden.value}
      <h1>Naviate to your settings page to setup gradegetter</h1>
      <!--  <h1>GradeGetter under some heat rn...</h1> -->
    {:else if Object.keys(grades.value).length === 0}
      <p>Loading...</p>
    {:else}
      <div class="grades">
        {#each Object.entries(grades.value) as [subject, scores]}
          <h2>{subject}</h2>
          <ul>
            {#each scores as score, i}
              <li>
                <span>Q{i + 1}</span>
                <span class={score !== null ? "score" : "na"}>
                  {score !== null ? score.toFixed(2) : "N/A"}
                </span>
              </li>
            {/each}
          </ul>
        {/each}
      </div>
    {/if}
  {:else}
    <h1>To use GradeGetter, Register an account</h1>
  {/if}
</main>

<style>
  .grades {
    max-width: var(--column-width);
    margin: 2rem auto;
    padding: 1.5rem;
    background-color: rgba(255, 255, 255, 0.02);
    border: 1px solid var(--color-border);
    border-radius: 1rem;
    box-shadow: 0 4px 12px rgba(0, 0, 0, 0.3);
    backdrop-filter: blur(6px);
  }

  .grades h2 {
    margin-top: 2rem;
    font-size: 1.4rem;
    color: var(--color-theme-2);
    border-bottom: 1px solid var(--color-border);
    padding-bottom: 0.5rem;
  }

  .grades ul {
    list-style: none;
    padding-left: 0;
    margin-top: 1rem;
  }

  .grades li {
    display: flex;
    justify-content: space-between;
    align-items: center;
    background-color: rgba(255, 255, 255, 0.04);
    padding: 0.75rem 1rem;
    margin-bottom: 0.5rem;
    border-radius: 0.5rem;
    color: var(--color-text);
    transition: background 0.2s ease;
  }

  .grades li:hover {
    background-color: rgba(255, 255, 255, 0.08);
  }

  .grades li span.score {
    font-weight: 600;
    color: var(--color-theme-1);
    font-family: var(--font-mono);
  }

  .grades li span.na {
    color: var(--color-subtle-text);
    font-style: italic;
    font-family: var(--font-mono);
  }

  @media (max-width: 600px) {
    .grades li {
      flex-direction: column;
      align-items: flex-start;
      gap: 0.25rem;
    }
  }
</style>
