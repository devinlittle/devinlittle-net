<script lang="ts">
  import {
    auth,
    type ServiceName,
    type UserRole,
  } from "$lib/utils/auth.svelte";

  const PodcastSchoolProjectService: ServiceName = "podcastschoolproject";

  function getPodcastRole(roles: UserRole): UserRole {
    return roles[PodcastSchoolProjectService] ?? "user";
  }

  let allowed_to_view = $state(false);

  $effect(() => {
    if (auth.ready) {
      if (getPodcastRole(auth.roles) != "user") {
        allowed_to_view = true;
      }
    }
  });
</script>

{#if allowed_to_view}
  <h1>Imagine the Podcasts being here</h1>

  <img src="podcastproject/Logo.webp" alt="" />
{:else}
  <h1>You aren't supposed to be here</h1>
{/if}
