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

  let allowed_to_view = $state(true);

  $effect(() => {
    if (auth.ready) {
      if (getPodcastRole(auth.roles) != "user") {
        allowed_to_view = true;
      }
    }
  });

  let activeTab = $state<number>(1);

  const podcast = {
    title: "Meow Meow Meow I'm a Kitty I'm a Cat",
    logoUrl: "podcastproject/Logo.webp",
    description: `ChatGBT wrote this. DISCLAIMER!!! ChatGBT was not used to create this
        biography, it was a June Fools joke. Ever wondered which big cat is the
        best? Tune in to my latest financial crypto stockings, the Meow Meow
        Meow I’m a Kitty Kitty I’m a Cat podcats! Wait... I didn’t just say
        podCATS did I? LOL!! Copyright Ethan Rodgers Anyway, join our hosts as
        they rank the elusive big cats on their fierceness (purr henny), horror
        (It’s Kitty!!) and last but definitely not least, CUTENESS!!!! (Pawbs
        >-o) Copyright Ethan Rodgers This is such a serious topic that only
        geniuses with the best rhinos could debate it. Who are these prodigies,
        you may ask? Well I’ll tell you who these virtuosos of literature and
        cats are. These masterminds are none other than Ethan James Rodgers and
        Devin Nicholas Litter. Ethan, well, his life story is a real doozy.
        Often referred to as a whistling dixie by his peers; Ethan James Rodgers
        spent most of his young life aloof in the beauty of nature. Growing up
        in Oregon, this was quite easy to do. Imagine this--verdant trees in
        every direction, rushing rivers sweeping you off your feet (this
        happened to him in the summer between his 8th and 9th years of school.
        He was walking in the river alone for whatever reason and got dragged by
        an undercurrent. He, however, had rafted the river before and knew there
        were embankments on the other side. Despite this, he would not have been
        able to climb over the wall of dirt trapping him had it not been for his
        Great uncles coming to his aid.) and dragging you off to cliffs rivaling
        Clifford himself! Ah, just Copyright Ethan Rodgers recalling these
        memories induces such elation in him... Regardless! He is a cat expert
        whose favorite animal has been every cat ever since he read Warrior Cats
        (suggested by his mother) in elementary school. Next up is Devin
        Nicholas Copyright Ethan Rodgers Little. He’s an adventurous person who
        likes to dress up as a cat secretly but no one will ever read this
        because there are so many comments. “Idk I feel like that’s it that’s
        all I do” he says. I’ll pass the mic over to Ethan James Rodgers so he
        can hopefully fill in the blanks. Hi this is Ethan James Rodgers and
        he’s mostly right but he is also an expert on this topic as he was
        obsessed with sneps since he was like elementary school after his 4th
        grade teacher told him about wikipedia. Correction he was obsessed with
        sneps after MacOS Snowleopard because he was a big MacOS fan and the UI
        looked cool and he really liked the cat and he was like “wow I like
        that”. And yeah he’s chill or whatever so there’s that idk much about
        him he’s kinda mysterious. Well would you look at that! Two rich, hot,
        and expert hosts! Surely, they will bring much enlightenment to the
        viewers of this podcast, right? I definitely Copyright Ethan Rodgers
        think so. Make sure you tune in, and don’t forget to like, subscribe,
        and donate to our GoFundMe to keep this podcast going!`,
    episodes: [
      {
        id: 1,
        title: "Episode 1: Leopards Vs Snow Leopards",
        embedId: "HJ0KDqChJJ4",
        notes: "",
      },
      {
        id: 2,
        title: "Episode 2: Tiger Vs Cougar",
        embedId: "cxx82UrFgM0",
        notes: "",
      },
      {
        id: 3,
        title: "Episode 3: Lion Vs Jaguar Vs Cheetah",
        embedId: "MB43Nw4RhS0",
        notes: "",
      },
    ],
  };
</script>

{#if allowed_to_view}
  <div class="page">
    <header class="panel">
      <div class="top-row">
        <div>
          <h1>{podcast.title}</h1>
          <p class="subtitle">A Devin James Production</p>
        </div>
        <img
          src={podcast.logoUrl}
          alt="Podcast Series Logo"
          style="max-width: 120px; border-radius: 0.5rem; border: 1px solid var(--color-border);"
        />
      </div>
      <hr class="divider" />
      <p style="line-height: 1.6; color: var(--color-text);">
        {podcast.description}
      </p>
    </header>

    <main class="panel">
      <h2>Podcast Episodes</h2>

      <div class="tabs">
        {#each podcast.episodes as ep}
          <button
            type="button"
            class="tab {activeTab === ep.id ? 'active' : ''}"
            on:click={() => (activeTab = ep.id)}
          >
            Ep {ep.id}
            <span class="count">Audio</span>
          </button>
        {/each}
      </div>

      {#each podcast.episodes as ep}
        {#if activeTab === ep.id}
          <div
            style="display: flex; flex-direction: column; gap: 1rem; margin-top: 0.5rem;"
          >
            <h3 style="color: var(--color-theme-1); margin: 0;">{ep.title}</h3>

            <div
              style="position: relative; padding-bottom: 56.25%; height: 0; overflow: hidden; border-radius: 0.75rem; border: 1px solid var(--color-border);"
            >
              <iframe
                src="https://www.youtube-nocookie.com/embed/{ep.embedId}"
                title="YouTube video player"
                frameborder="0"
                style="width: 100%; aspect-ratio: 16 / 9; border: 1px solid var(--color-border); border-radius: 0.75rem;"
                allow="accelerometer;
                autoplay; clipboard-write; encrypted-media; gyroscope; picture-in-picture;
                web-share"
                referrerpolicy="strict-origin-when-cross-origin"
                allowfullscreen
              ></iframe>
            </div>
          </div>
        {/if}
      {/each}
    </main>
  </div>
{:else}
  <div class="page">
    <div class="panel empty">
      <h1>You aren't supposed to be here</h1>
    </div>
  </div>
{/if}
