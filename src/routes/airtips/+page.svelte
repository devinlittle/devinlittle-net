<script>
  import { onMount } from "svelte";

  const apiUrl = "api.devinlittle.net";

  let token = $state("");
  let songs = $state([]);
  let loading = $state(true);
  let loggedIn = $state(false);

  function timeAgo(unix) {
    const now = Date.now() / 1000;
    const diff = now - unix;

    if (diff < 60) return "just now";
    if (diff < 3600)
      return `${Math.floor(diff / 60)} minute${diff >= 120 ? "s" : ""} ago`;
    if (diff < 86400)
      return `${Math.floor(diff / 3600)} hour${diff >= 7200 ? "s" : ""} ago`;
    if (diff < 604800)
      return `${Math.floor(diff / 86400)} day${diff >= 172800 ? "s" : ""} ago`;
    if (diff < 31536000)
      return `${Math.floor(diff / 604800)} week${diff >= 1209600 ? "s" : ""} ago`;
    return "over a year ago";
  }

  function freshness(unix) {
    const now = Date.now() / 1000;
    const diff = now - unix;
    const maxAge = 7 * 24 * 60 * 60; // 7 days
    return Math.max(0, 1 - diff / maxAge);
  }

  const init = async () => {
    token = localStorage.getItem("token") || "";
    loggedIn = !!token;
    if (loggedIn) await fetchSongs();
    loading = false;
  };

  const fetchSongs = async () => {
    try {
      loading = true;
      const res = await fetch(`https://${apiUrl}/airtips/recently_played/1`, {
        headers: { Authorization: `Bearer ${token}` },
      });
      if (!res.ok) throw new Error("Failed");
      const data = await res.json();
      songs = data.songs || [];
    } catch (err) {
      console.error(err);
    } finally {
      loading = false;
    }
  };

  onMount(init);
</script>

<svelte:head>
  <title>Recently Played • AirTips</title>
</svelte:head>

<div class="text-column">
  <h1>Recently Played</h1>

  {#if !loggedIn}
    <p style="text-align:center; opacity:0.8;">
      Log in to see your music history
    </p>
  {:else if loading}
    <p style="text-align:center; opacity:0.7;">Loading...</p>
  {:else if songs.length === 0}
    <p style="text-align:center; opacity:0.7;">No songs yet</p>
  {:else}
    <div class="songs-list">
      {#each songs as song, i}
        {@const fresh = freshness(song.playedAt)}
        <div
          class="song-item"
          style:--delay={i * 0.05 + "s"}
          style:--freshness={fresh}
        >
          <img src={song.imageSrc} alt={song.title} />

          <div class="song-info">
            <div class="title">{song.title}</div>
            <div class="artist">{song.artist}</div>
            {#if song.album}
              <div class="album">{song.album}</div>
            {/if}
            <div class="played">Played {timeAgo(song.playedAt)}</div>
          </div>
        </div>

        <br />
      {/each}
    </div>
  {/if}
</div>

<style>
  .song-item {
    position: relative;
    display: grid;
    grid-template-columns: 80px 1fr auto;
    gap: 1rem;
    align-items: center;
    padding: 1.2rem;
    background: rgba(255, 255, 255, 0.03);
    border: 1px solid var(--color-border);
    border-radius: 0.9rem;
    backdrop-filter: blur(10px);
    overflow: hidden;
    animation: fadeInUp 0.7s ease-out backwards;
    animation-delay: var(--delay);
    transition: all 0.35s ease;
  }

  .song-item:hover {
    background: rgba(255, 255, 255, 0.08);
    border-color: var(--color-theme-1);
    transform: translateY(-3px);
    box-shadow: 0 12px 28px rgba(126, 156, 255, 0.2);
  }

  .song-item img {
    width: 80px;
    height: 80px;
    object-fit: cover;
    border-radius: 0.7rem;
    box-shadow: 0 4px 16px rgba(0, 0, 0, 0.4);
  }

  .song-info {
    display: flex;
    flex-direction: column;
    gap: 0.4rem;
    min-width: 0;
  }

  .title {
    font-weight: 600;
    font-size: 1.15rem;
    color: var(--color-text);
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }
  .artist {
    color: var(--color-theme-2);
    font-size: 0.98rem;
  }
  .album {
    color: var(--color-subtle-text);
    font-size: 0.88rem;
    font-style: italic;
  }
  .played {
    font-size: 0.82rem;
    color: var(--color-theme-1);
    font-weight: 500;
    opacity: 0.9;
  }

  @keyframes fadeInUp {
    from {
      opacity: 0;
      transform: translateY(25px);
    }
    to {
      opacity: 1;
      transform: translateY(0);
    }
  }

  @media (max-width: 600px) {
    .song-item {
      grid-template-columns: 70px 1fr;
      padding: 1rem;
    }
  }
</style>
