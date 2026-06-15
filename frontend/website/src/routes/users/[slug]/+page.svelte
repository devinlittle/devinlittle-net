<script lang="ts">
  import { auth } from "$lib/utils/auth.svelte";
  import type { PageProps } from "./$types";

  let { data }: PageProps = $props();
</script>

<svelte:head>
  <title>DevinLittle.net - User Page</title>
  <meta name="description" content="Devin's gradegetter duh" />
</svelte:head>

<main>
  {#if auth.ready}
    <h1>imagine user data here all formated nicely</h1>
    <!--> 
[ username    <MessageButton> (could be request if user has that option)]
[ bio ... pubkey]               [section of user own contact info (inside of sqlite table, like  a contact page)]
                                [IF ADMIN ROLE {owen or devin}, CAN SEE USER ROLES]
[chats w/ user section]
<!-->
  {:else}
    <h1>Need to be logged in to view user data.</h1>
  {/if}
</main>
