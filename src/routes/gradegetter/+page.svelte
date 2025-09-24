<script>
  import { onMount, onDestroy } from "svelte";
  import { goto } from "$app/navigation";
  let apiUrl = "api.devinlittle.net";

  let LoggedIn = $state(false);
  let token;
  let grades = $state({});

  let load = async () => {
    if (localStorage.getItem("token") === null) {
      LoggedIn = false;
    } else {
      LoggedIn = true;
      token = localStorage.getItem("token");
    }
  };

  let fetchGrades = async () => {
    if (LoggedIn) {
      const response = await fetch(`https://${apiUrl}:3000/grades`, {
        method: "GET",
        headers: {
          "Content-Type": "application/json",
          Authorization: `Bearer ${token}`,
        },
      });

      if (!response.ok) {
        console.error("Failed to fetch grades");
        //     throw new Error(`Schoology registration failed: ${msg}`);
      } else {
        const newGrades = await response.json();

        for (const subject in newGrades) {
          grades[subject] = newGrades[subject];
        }

        for (const subject in grades) {
          if (!(subject in newGrades)) {
            delete grades[subject];
          }
        }

        console.log("ur in baby!");
      } // if end here
    }
  };

  async function logOut(event) {
    localStorage.removeItem("token");
    LoggedIn = false;
    goto("/gradegetter");
  }

  onMount(() => {
    load();
    fetchGrades();

    const interval = setInterval(() => {
      fetchGrades();
    }, 5000); // every 5 seconds

    onDestroy(() => {
      clearInterval(interval);
    });
  });
</script>

{#if LoggedIn}
  <button onclick={logOut}>Log Out</button>
  {#if Object.keys(grades).length === 0}
    <p>Loading...</p>
  {:else}
    {#each Object.entries(grades) as [subject, scores]}
      <h2>{subject}</h2>
      <ul>
        {#each scores as score, i}
          <li>Q{i + 1}: {score !== null ? score.toFixed(2) : "N/A"}</li>
        {/each}
      </ul>
    {/each}
  {/if}
{:else}
  <h1>Logged Out...</h1>
{/if}
