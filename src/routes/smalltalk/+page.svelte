<script lang="ts">
  import { auth } from "$lib/utils/auth.svelte";

  // TODO:
  // first load all conversations from sqlite
  // then do checks to sync sqlite with real smalltalk_db
  // if smalltalk_db got new shit:
  // > load conversation info based off jwt
  // > else just keep the first prefetched data
  //
</script>

{#if auth.ready}
  <h1>Imagine the SmallTalk UI here</h1>
{:else}{/if}
