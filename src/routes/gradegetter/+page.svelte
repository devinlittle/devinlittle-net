<script>
  import { onMount, onDestroy } from "svelte";
  import { goto } from "$app/navigation";
  import { API_URL, auth, authFetch } from "$lib/utils/auth.svelte.js";

  let grades = $state({});
  let forbiddon = $state(false);

  let fetchGrades = async () => {
    const response = await authFetch(`${API_URL}/gradegetter/grades`, {
      method: "GET",
      headers: {
        "Content-Type": "application/json",
      },
    });

    if (!response.ok) {
      forbiddon = true;
      if (response.status === 403) {
        // Response is Forbiddon
        forbiddon = true;
        return;
      }
      console.error("Failed to fetch grades");
      return;
    }

    forbiddon = false;

    const newGrades = await response.json();
    for (const subject in newGrades) grades[subject] = newGrades[subject];
    for (const subject in grades) {
      if (!(subject in newGrades)) delete grades[subject];
    }
  };

  $effect(() => {
    if (!auth.ready) return;

    fetchGrades();

    const interval = setInterval(fetchGrades, 5000);

    return () => clearInterval(interval);
  });
</script>

<svelte:head>
  <title>DevinLittle.net - GradeGetter</title>
  <meta name="description" content="Devin's gradegetter duh" />
</svelte:head>

<main>
  {#if auth.ready}
    {#if forbiddon}
      <h1>GradeGetter under some heat rn...</h1>
    {:else if Object.keys(grades).length === 0}
      <p>Loading...</p>
    {:else}
      <div class="grades">
        {#each Object.entries(grades) as [subject, scores]}
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
    <h1>GradeGetter under some heat rn...</h1>
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
