<script lang="ts">
  import { auth } from "$lib/utils/auth.svelte";
</script>

{#if auth.ready}
  <!-->
show seach bar for users
<!-->
  <h1>imagine a search bar here</h1>
{:else}
  <h1>Need to be logged in to search for users.</h1>
{/if}
