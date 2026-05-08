<script lang="ts">
  import {
    nanopass,
    fetchListings,
    registerHostedFile,
    sendNanoPass,
    unregisterHostedFile,
  } from "$lib/utils/nanopass.svelte";
  import { auth, API_URL, authFetch } from "$lib/utils/auth.svelte";
  import type { FileListing, Visibility } from "$lib/utils/nanopass.types";
  import { formatBytes } from "$lib/utils/notifications.svelte";
  import { beforeNavigate } from "$app/navigation";
  import { db_exec, db_run, upsert_contact } from "$lib/utils/sqlite.svelte";

  let activeTab = $state<"mine" | "public" | "forme">("mine");

  // --- upload state ---
  let showFileUploadModal = $state(false);
  let showEditFileModal = $state(false);
  let pendingFile = $state<File | null>(null);
  let selectedVisibility = $state<"Private" | "Public" | "Restricted">(
    "Private",
  );
  let selectedAutoAcceptState = $state(false);
  let uploading = $state(false);
  let fileInput: HTMLInputElement;

  const myListings = $derived(
    nanopass.listings.filter((l) => l.owner_id === auth.id),
  );
  const publicListings = $derived(
    nanopass.listings.filter((l) => l.visibility.type === "Public"),
  );
  const forMeListings = $derived(
    nanopass.listings.filter(
      (l) =>
        l.visibility.type === "Restricted" &&
        l.visibility.allowlist.includes(auth.id ?? "") &&
        l.owner_id !== auth.id,
    ),
  );
  const activeListings = $derived(
    activeTab === "mine"
      ? myListings
      : activeTab === "public"
        ? publicListings
        : forMeListings,
  );

  function formatDate(iso: string): string {
    const d = new Date(parseInt(iso) * 1000);

    const diff = (Date.now() - d.getTime()) / 1000;

    if (diff < 60) return "just now";
    if (diff < 3600) return `${Math.floor(diff / 60)}m ago`;
    if (diff < 86400) return `${Math.floor(diff / 3600)}h ago`;

    return d.toLocaleDateString();
  }

  function truncateId(id: string): string {
    return id.slice(0, 8) + "...";
  }

  function visibilityLabel(listing: FileListing): string {
    return listing.visibility.type.toLowerCase();
  }

  function openFilePicker() {
    fileInput.click();
  }

  function onFileChosen(e: Event) {
    const input = e.target as HTMLInputElement;
    const file = input.files?.[0];
    if (!file) return;
    pendingFile = file;
    selectedVisibility = "Private";
    showFileUploadModal = true;
    input.value = "";
  }

  async function confirmListing() {
    if (!pendingFile || !auth.session_id || !auth.accessToken) return;
    uploading = true;
    const visibility: Visibility = {
      type: selectedVisibility,
      ...(selectedVisibility === "Restricted" && { allowlist: [] }),
    } as Visibility;
    try {
      const res = await authFetch(`${API_URL}/nanopass/listings`, {
        method: "POST",
        headers: {
          "Content-Type": "application/json",
          Authorization: `Bearer ${auth.accessToken}`,
        },
        body: JSON.stringify({
          owner_id: auth.id,
          session_id: auth.session_id,
          filename: pendingFile.name,
          size_bytes: pendingFile.size,
          created_at: Math.round(Date.now() / 1000),
          mime_type: pendingFile.type || "application/octet-stream",
          visibility,
          auto_accept: selectedAutoAcceptState,
        }),
      });
      if (res.ok) {
        const listing: FileListing = await res.json();
        registerHostedFile(listing.id, pendingFile);
        showFileUploadModal = false;
        pendingFile = null;
        activeTab = "mine";
      }
    } finally {
      uploading = false;
    }
  }

  async function removeListing(id: string) {
    let res = await authFetch(`${API_URL}/nanopass/listings`, {
      method: "DELETE",
      headers: { "Content-Type": "application/json" },
      credentials: "include",
      body: JSON.stringify({
        listing_id: id,
      }),
    });
    if (res.ok) {
      unregisterHostedFile(id);
    }
  }

  function cancelModal() {
    showFileUploadModal = false;
    pendingFile = null;
  }

  let pendingEditListing: FileListing = $state(null);
  let pendingEditUsersList = $state([]);

  let search = $state("");
  let search_results = $state([]);
  let searching = $state(false);
  let selected = $state([]);
  let show_search = $state(false);

  async function showEditFile() {
    if (pendingEditListing.visibility.type === "Restricted") {
      const allowlist = pendingEditListing.visibility.allowlist;

      const resolved = {};
      const missing = [];

      for (const id of allowlist) {
        const rows = await db_exec(
          `SELECT user_id, username FROM contacts WHERE user_id = ?`,
          [id],
        );
        if (rows[0]) {
          resolved[id] = rows[0];
        } else {
          missing.push(id);
        }
      }

      if (missing.length > 0) {
        const res = await authFetch(`${API_URL}/auth/users/by-ids`, {
          method: "POST",
          headers: {
            "Content-Type": "application/json",
            Authorization: `Bearer ${auth.accessToken}`,
          },
          body: JSON.stringify({ ids: missing }),
        });

        if (res.ok) {
          const users = await res.json();
          for (const user of users) {
            upsert_contact({
              user_id: user.id,
              username: user.username,
              public_key: user.public_key,
              last_seen: user.last_seen,
            });
            resolved[user.id] = user;
          }
        }
      }

      pendingEditUsersList = allowlist;
      selected = allowlist.map((id) => resolved[id] ?? { id, username: id });
    }
    showEditFileModal = true;
  }

  async function confirmEditFile() {
    const visibility: Visibility = {
      type: selectedVisibility,
      ...(selectedVisibility === "Restricted" && {
        allowlist: pendingEditUsersList,
      }),
    } as Visibility;

    const res = await authFetch(`${API_URL}/nanopass/listings`, {
      method: "PATCH",
      headers: {
        "Content-Type": "application/json",
        Authorization: `Bearer ${auth.accessToken}`,
      },
      body: JSON.stringify({
        id: pendingEditListing.id,
        owner_id: pendingEditListing.owner_id,
        owner_username: pendingEditListing.owner_username,
        session_id: pendingEditListing.session_id,
        filename: pendingEditListing.filename,
        size_bytes: pendingEditListing.size_bytes,
        created_at: pendingEditListing.created_at,
        mime_type: pendingEditListing.mime_type,
        visibility: visibility,
        auto_accept: selectedAutoAcceptState,
      }),
    });

    if (res.ok) {
      pendingEditListing = null;
      pendingEditUsersList = [];
      selected = [];
      showEditFileModal = false;
    }
  }

  function cancelEditFile() {
    pendingEditListing = null;
    pendingEditUsersList = [];
    selected = [];
    showEditFileModal = false;
  }

  async function searchUsers(q) {
    if (!q.trim()) {
      search_results = [];
      return;
    }
    searching = true;
    try {
      const res = await authFetch(
        `${API_URL}/auth/users/search?q=${encodeURIComponent(q)}`,
      );
      if (res.ok) search_results = await res.json();
    } finally {
      searching = false;
    }
  }

  let search_timeout;
  function onSearchInput() {
    clearTimeout(search_timeout);
    search_timeout = setTimeout(() => searchUsers(search), 300);
  }

  function selectUser(user) {
    if (selected.find((s) => s.id === user.id)) return;
    selected = [...selected, user];
    pendingEditUsersList = [...pendingEditUsersList, user.id];
    search = "";
    search_results = [];
    show_search = false;
  }

  function removeUser(id) {
    pendingEditUsersList = pendingEditUsersList.filter((i) => i !== id);
    selected = selected.filter((s) => s.id !== id);
  }

  function initiateTransfer(listing: FileListing) {
    // INFO: FileQuery -> FileQueryResponse -> TransferRequest
    sendNanoPass(
      {
        type: "FileQuery",
        listing_id: listing.id,
        requester_session_id: auth.session_id,
      },
      null,
      listing.owner_id,
    );

    console.log("initiating transfer for", listing.id);
  }

  let isTransferring = $derived(
    Object.values(nanopass.transferProgress).some(
      (p) => p !== null && p > 0 && p < 1,
    ),
  );

  let dragging = $state(false);

  function onDragOver(e: DragEvent) {
    e.preventDefault();
    dragging = true;
  }

  function onDragLeave(e: DragEvent) {
    if (e.relatedTarget === null) dragging = false;
  }

  function onDrop(e: DragEvent) {
    e.preventDefault();
    dragging = false;
    const file = e.dataTransfer?.files?.[0];
    if (!file) return;
    pendingFile = file;
    selectedVisibility = "Private";
    showFileUploadModal = true;
  }

  beforeNavigate(({ cancel }) => {
    if (isTransferring) {
      if (
        !confirm(
          "A file transfer is in progress. Leaving now will cancel it. Continue?",
        )
      ) {
        cancel();
      }
    }
  });

  $effect(() => {
    if (!auth.ready) return;

    fetchListings();
  });
</script>

<svelte:head>
  <title>DevinLittle.net - NanoPass</title>
  <meta
    name="description"
    content="NanoPass, secure peer to peer file sharing"
  />
</svelte:head>

<input
  bind:this={fileInput}
  type="file"
  style="display:none"
  onchange={onFileChosen}
/>

{#if auth.ready}
  {#if showFileUploadModal && pendingFile}
    <div class="modal-backdrop" onclick={cancelModal}>
      <div class="modal" onclick={(e) => e.stopPropagation()}>
        <div class="modal-header">
          <span>share file</span>
          <button class="modal-close" onclick={cancelModal}>✕</button>
        </div>
        <div class="modal-body">
          <div class="file-preview">
            <span class="file-preview-name">{pendingFile.name}</span>
            <span class="file-preview-size"
              >{formatBytes(pendingFile.size)}</span
            >
          </div>
          <div class="field-row">
            <label>visibility</label>
            <div class="vis-options">
              {#each ["Private", "Public", "Restricted"] as v}
                <button
                  class="vis-btn"
                  class:active={selectedVisibility === v}
                  onclick={() =>
                    (selectedVisibility = v as typeof selectedVisibility)}
                  >{v.toLowerCase()}</button
                >
              {/each}
            </div>
            <p class="vis-hint">
              {#if selectedVisibility === "Private"}
                only your own devices can access this file
              {:else if selectedVisibility === "Public"}
                anyone on NanoPass can access this file
              {:else}
                only users on this allowlist can access this file
              {/if}
            </p>
          </div>

          <div class="field-row">
            <label>
              <input type="checkbox" bind:checked={selectedAutoAcceptState} />
              Auto Accept?
            </label>
            <p class="vis-hint">
              Enabling this will auto download the file and <strong
                >WILL NOT</strong
              > prompt you for a confirmation.
            </p>
          </div>
        </div>
        <div class="modal-footer">
          <button class="btn-ghost" onclick={cancelModal}>cancel</button>
          <button
            class="btn-confirm"
            onclick={confirmListing}
            disabled={uploading}
          >
            {uploading ? "listing..." : "list file"}
          </button>
        </div>
      </div>
    </div>
  {/if}

  {#if showEditFileModal && pendingEditListing}
    <div class="modal-backdrop" onclick={cancelEditFile}>
      <div class="modal" onclick={(e) => e.stopPropagation()}>
        <div class="modal-header">
          <span>Edit file properties</span>
          <button class="modal-close" onclick={cancelEditFile}>✕</button>
        </div>
        <div class="modal-body">
          <div class="file-preview">
            <span class="file-preview-name">{pendingEditListing.filename}</span>
            <span class="file-preview-size"
              >{formatBytes(pendingEditListing.size_bytes)}</span
            >
          </div>
          <div class="field-row">
            <label>visibility</label>
            <div class="vis-options">
              {#each ["Private", "Public", "Restricted"] as v}
                <button
                  class="vis-btn"
                  class:active={selectedVisibility === v}
                  onclick={() =>
                    (selectedVisibility = v as typeof selectedVisibility)}
                  >{v.toLowerCase()}</button
                >
              {/each}
            </div>
            {#if selectedVisibility === "Restricted"}
              <div class="search-wrap">
                <input
                  type="text"
                  bind:value={search}
                  oninput={onSearchInput}
                  placeholder="search users..."
                  class="search-input"
                />
                {#if searching}
                  <div class="search-status">searching...</div>
                {:else if search && search_results.length === 0}
                  <div class="search-status">no users found</div>
                {:else}
                  {#each search_results as user}
                    <button
                      class="search-result"
                      onclick={() => selectUser(user)}
                    >
                      <div class="user-avatar">
                        {user.username.slice(0, 2).toUpperCase()}
                      </div>
                      <div class="user-info">
                        <span class="user-name">{user.username}</span>
                        {#if user.bio}<span class="user-bio">{user.bio}</span
                          >{/if}
                      </div>
                    </button>
                  {/each}
                {/if}
              </div>

              <br />

              {#each selected as user}
                <div class="pill">
                  <span>{user.username}</span>
                  <button
                    class="pill-remove"
                    onclick={() => removeUser(user.id)}
                  >
                    <svg
                      width="10"
                      height="10"
                      viewBox="0 0 16 16"
                      fill="none"
                      stroke="currentColor"
                      stroke-width="2.5"
                    >
                      <path d="M2 2l12 12M14 2L2 14" />
                    </svg>
                  </button>
                </div>
              {/each}
            {/if}

            <p class="vis-hint">
              {#if selectedVisibility === "Private"}
                only your own devices can request this file
              {:else if selectedVisibility === "Public"}
                anyone on NanoPass can see and request this file
              {:else}
                only users you allowlist can see this file
              {/if}
            </p>
          </div>

          <div class="field-row">
            <label>
              <input type="checkbox" bind:checked={selectedAutoAcceptState} />
              Auto Accept?
            </label>
            <p class="vis-hint">
              Enabling this will auto download the file and <strong
                >WILL NOT</strong
              > prompt you for a confirmation.
            </p>
          </div>
        </div>
        <div class="modal-footer">
          <button class="btn-ghost" onclick={cancelEditFile}>cancel</button>
          <button
            class="btn-confirm"
            onclick={confirmEditFile}
            disabled={uploading}
          >
            {uploading ? "applying..." : "apply changes"}
          </button>
        </div>
      </div>
    </div>
  {/if}

  <div
    class="page"
    ondragover={onDragOver}
    ondragleave={onDragLeave}
    ondrop={onDrop}
  >
    {#if dragging}
      <div class="drop-overlay">
        <div class="drop-box">
          <svg
            width="32"
            height="32"
            viewBox="0 0 24 24"
            fill="none"
            stroke="currentColor"
            stroke-width="1.5"
          >
            <path d="M21 15v4a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2v-4" />
            <polyline points="17 8 12 3 7 8" />
            <line x1="12" y1="3" x2="12" y2="15" />
          </svg>
          drop to share
        </div>
      </div>
    {/if}

    <div class="top-row">
      <div>
        <h1>NanoPass</h1>
        <p class="subtitle">peer-to-peer file sharing</p>
      </div>
      <button class="btn-primary" onclick={openFilePicker}
        >+ share a file</button
      >
    </div>

    <div class="panel">
      <div class="tabs">
        <button
          class="tab"
          class:active={activeTab === "mine"}
          onclick={() => (activeTab = "mine")}
        >
          my listings
          {#if myListings.length > 0}<span class="count"
              >{myListings.length}</span
            >{/if}
        </button>
        <button
          class="tab"
          class:active={activeTab === "public"}
          onclick={() => (activeTab = "public")}
        >
          public
          {#if publicListings.length > 0}<span class="count"
              >{publicListings.length}</span
            >{/if}
        </button>
        <button
          class="tab"
          class:active={activeTab === "forme"}
          onclick={() => (activeTab = "forme")}
        >
          for me
          {#if forMeListings.length > 0}<span class="count"
              >{forMeListings.length}</span
            >{/if}
        </button>
      </div>

      {#if activeListings.length === 0}
        <div class="empty">
          <p>
            {#if activeTab === "mine"}
              you haven't listed any files yet —
              <button class="inline-link" onclick={openFilePicker}
                >share one now</button
              >
            {:else if activeTab === "public"}
              no public files available right now
            {:else}
              no files shared with you yet
            {/if}
          </p>
        </div>
      {:else}
        <div class="grid">
          {#each activeListings as listing (listing.id)}
            {@const progress = nanopass.transferProgress[listing.id] ?? null}
            <div class="card">
              <div class="card-top">
                <span class="filename" title={listing.filename}
                  >{listing.filename}</span
                >
                <span
                  class="badge badge-{listing.visibility.type.toLowerCase()}"
                >
                  {visibilityLabel(listing)}
                </span>
                {#if activeTab === "mine"}
                  <button
                    class="remove-btn"
                    onclick={() => {
                      pendingEditListing = listing;
                      showEditFile();
                    }}>Edit</button
                  >

                  <button
                    class="remove-btn"
                    onclick={() => removeListing(listing.id)}
                    title="remove listing"
                  >
                    <svg
                      width="12"
                      height="12"
                      viewBox="0 0 16 16"
                      fill="none"
                      stroke="currentColor"
                      stroke-width="2.5"
                    >
                      <path d="M2 2l12 12M14 2L2 14" />
                    </svg>
                  </button>
                {/if}
              </div>
              <div class="meta">
                <div class="meta-row">
                  <span class="meta-label">size</span>
                  <span class="meta-value"
                    >{formatBytes(listing.size_bytes)}</span
                  >
                </div>
                <div class="meta-row">
                  <span class="meta-label">type</span>
                  <span class="meta-value">{listing.mime_type}</span>
                </div>
                <div class="meta-row">
                  <span class="meta-label">listed</span>
                  <span class="meta-value"
                    >{formatDate(listing.created_at)}</span
                  >
                </div>
                <div class="meta-row">
                  <span class="meta-label"
                    >{listing.owner_id === auth.id ? "device" : "from"}</span
                  >
                  <span class="meta-value">
                    {(() => {
                      // if listing wasn't made but current device
                      if (listing.owner_id !== auth.id)
                        return listing.owner_username;

                      switch (true) {
                        case listing.session_id === auth.session_id:
                          return "Current Device";
                        default:
                          // TODO:: make it show device os, example:
                          // device: Windows
                          return truncateId(listing.session_id);
                      }
                    })()}
                  </span>
                </div>
              </div>
              <hr class="divider" />
              <button
                class="btn-download"
                class:downloading={progress !== null && progress < 1}
                class:done={progress !== null && progress >= 1}
                disabled={progress !== null && progress < 1}
                onclick={() => initiateTransfer(listing)}
              >
                {#if progress !== null && progress >= 1}
                  <svg
                    width="14"
                    height="14"
                    viewBox="0 0 16 16"
                    fill="none"
                    stroke="currentColor"
                    stroke-width="2"
                  >
                    <path d="M2 8l4 4 8-8" />
                  </svg>
                  downloaded
                {:else if progress !== null}
                  <span class="btn-progress-wrap">
                    <span
                      class="btn-progress-fill"
                      style="width: {Math.round(progress * 100)}%"
                    ></span>
                    <span class="btn-progress-label"
                      >{Math.round(progress * 100)}%</span
                    >
                  </span>
                {:else}
                  <svg
                    width="14"
                    height="14"
                    viewBox="0 0 16 16"
                    fill="none"
                    stroke="currentColor"
                    stroke-width="1.5"
                  >
                    <path d="M8 2v8M5 7l3 3 3-3M3 13h10" />
                  </svg>
                  {listing.auto_accept === true
                    ? "download file"
                    : "request file"}
                {/if}
              </button>
            </div>
          {/each}

          {#if activeTab === "mine"}
            <button class="card card-add" onclick={openFilePicker}>
              <span class="add-icon">+</span>
              <span class="add-label">share a new file</span>
            </button>
          {/if}
        </div>
      {/if}
    </div>
  </div>
{:else}
  <h1>Log In to use NanoPass</h1>
{/if}

<style>
  .page {
    max-width: var(--column-width);
    margin: var(--column-margin-top) auto;
    padding: 0 1rem 4rem;
    display: flex;
    flex-direction: column;
    gap: 1.5rem;
  }
  .top-row {
    display: flex;
    align-items: center;
    justify-content: space-between;
    flex-wrap: wrap;
    gap: 1rem;
  }
  h1 {
    font-size: 2.5rem;
    font-weight: 600;
    color: var(--color-theme-1);
    margin: 0 0 0.25rem 0;
  }
  .subtitle {
    color: var(--color-subtle-text);
    font-size: 0.9rem;
    margin: 0;
  }
  .btn-primary {
    background: rgba(126, 156, 255, 0.12);
    border: 1px solid rgba(126, 156, 255, 0.35);
    color: var(--color-theme-1);
    padding: 0.5rem 1.25rem;
    border-radius: 0.4rem;
    font-size: 0.9rem;
    cursor: pointer;
    transition: all 0.2s ease;
  }
  .btn-primary:hover {
    background: rgba(126, 156, 255, 0.22);
    border-color: rgba(126, 156, 255, 0.55);
  }
  .panel {
    background: rgba(255, 255, 255, 0.02);
    border: 1px solid var(--color-border);
    border-radius: 1rem;
    padding: 1.5rem;
    backdrop-filter: blur(6px);
    box-shadow: 0 4px 12px rgba(0, 0, 0, 0.3);
    display: flex;
    flex-direction: column;
    gap: 1.25rem;
  }
  .tabs {
    display: flex;
    border-bottom: 1px solid var(--color-border);
    margin-bottom: -0.25rem;
  }
  .tab {
    display: flex;
    align-items: center;
    gap: 0.4rem;
    padding: 0.5rem 1.1rem;
    font-size: 0.875rem;
    cursor: pointer;
    color: var(--color-subtle-text);
    background: transparent;
    border: none;
    border-bottom: 2px solid transparent;
    margin-bottom: -1px;
    transition: all 0.2s ease;
    font-family: var(--font-body);
  }
  .tab.active {
    color: var(--color-theme-1);
    border-bottom-color: var(--color-theme-1);
  }
  .tab:hover:not(.active) {
    color: var(--color-text);
    background: rgba(255, 255, 255, 0.04);
  }
  .count {
    font-size: 0.7rem;
    background: rgba(126, 156, 255, 0.2);
    color: var(--color-theme-1);
    padding: 0.1rem 0.45rem;
    border-radius: 2rem;
    font-weight: 600;
  }
  .empty {
    text-align: center;
    padding: 3rem 1rem;
    color: var(--color-subtle-text);
    font-size: 0.9rem;
  }
  .inline-link {
    background: none;
    border: none;
    color: var(--color-theme-1);
    cursor: pointer;
    font-size: inherit;
    font-family: inherit;
    text-decoration: underline dotted;
    padding: 0;
  }
  .grid {
    display: grid;
    grid-template-columns: repeat(auto-fill, minmax(220px, 1fr));
    gap: 0.75rem;
  }
  .card {
    background: rgba(255, 255, 255, 0.03);
    border: 1px solid var(--color-border);
    border-radius: 0.75rem;
    padding: 1rem;
    display: flex;
    flex-direction: column;
    gap: 0.65rem;
    transition:
      background 0.2s ease,
      border-color 0.2s ease;
  }
  .card:hover {
    background: rgba(255, 255, 255, 0.05);
    border-color: rgba(126, 156, 255, 0.25);
  }
  .card-add {
    align-items: center;
    justify-content: center;
    min-height: 160px;
    border-style: dashed;
    border-color: rgba(255, 255, 255, 0.07);
    cursor: pointer;
    background: transparent;
    gap: 0.5rem;
  }
  .card-add:hover {
    border-color: rgba(126, 156, 255, 0.3);
    background: rgba(126, 156, 255, 0.03);
  }
  .add-icon {
    font-size: 1.5rem;
    color: rgba(126, 156, 255, 0.35);
  }
  .add-label {
    font-size: 0.82rem;
    color: var(--color-subtle-text);
  }

  .card-top {
    display: flex;
    align-items: flex-start;
    justify-content: space-between;
    gap: 0.5rem;
  }

  .card-top-right {
    display: flex;
    align-items: center;
    gap: 0.35rem;
    flex-shrink: 0;
  }

  .remove-btn {
    background: none;
    border: none;
    color: rgba(255, 100, 100, 0.4);
    cursor: pointer;
    padding: 0.15rem;
    display: flex;
    align-items: center;
    justify-content: center;
    border-radius: 0.25rem;
    transition: all 0.15s ease;
    line-height: 1;
  }

  .remove-btn:hover {
    color: #ff9090;
    background: rgba(255, 100, 100, 0.1);
  }

  .filename {
    font-size: 0.9rem;
    font-weight: 500;
    color: var(--color-text);
    word-break: break-all;
    line-height: 1.4;
  }
  .badge {
    font-size: 0.68rem;
    padding: 0.18rem 0.5rem;
    border-radius: 2rem;
    white-space: nowrap;
    font-weight: 500;
    flex-shrink: 0;
  }
  .badge-private {
    background: rgba(126, 156, 255, 0.15);
    color: var(--color-theme-1);
  }
  .badge-public {
    background: rgba(93, 202, 165, 0.15);
    color: #5dcaa5;
  }
  .badge-restricted {
    background: rgba(239, 159, 39, 0.15);
    color: #ef9f27;
  }
  .meta {
    display: flex;
    flex-direction: column;
    gap: 0.3rem;
  }
  .meta-row {
    display: flex;
    justify-content: space-between;
    align-items: center;
  }
  .meta-label {
    font-size: 0.75rem;
    color: var(--color-subtle-text);
  }
  .meta-value {
    font-size: 0.75rem;
    color: var(--color-text);
    font-family: var(--font-mono);
    max-width: 60%;
    text-align: right;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }
  .divider {
    border: none;
    border-top: 1px solid var(--color-border);
    margin: 0;
  }

  .btn-download {
    position: relative;
    overflow: hidden;
    width: 100%;
    background: rgba(126, 156, 255, 0.08);
    border: 1px solid rgba(126, 156, 255, 0.25);
    color: var(--color-theme-1);
    padding: 0.5rem 1rem;
    border-radius: 0.4rem;
    font-size: 0.85rem;
    cursor: pointer;
    font-family: var(--font-body);
    transition: all 0.2s ease;
    display: flex;
    align-items: center;
    justify-content: center;
    gap: 0.5rem;
  }
  .btn-download:hover:not(:disabled) {
    background: rgba(126, 156, 255, 0.18);
    border-color: rgba(126, 156, 255, 0.45);
  }
  .btn-download:disabled {
    cursor: not-allowed;
  }
  .btn-download.done {
    background: rgba(93, 202, 165, 0.12);
    border-color: rgba(93, 202, 165, 0.3);
    color: #5dcaa5;
  }

  .btn-progress-wrap {
    position: relative;
    width: 100%;
    display: flex;
    align-items: center;
    justify-content: center;
  }
  .btn-progress-fill {
    position: absolute;
    left: -1rem;
    top: -0.5rem;
    height: calc(100% + 1rem);
    background: rgba(126, 156, 255, 0.18);
    border-radius: 0.3rem;
    transition: width 0.15s ease;
    pointer-events: none;
  }
  .btn-progress-label {
    position: relative;
    font-family: var(--font-mono);
    font-size: 0.85rem;
    color: var(--color-theme-1);
    z-index: 1;
  }

  /* modal */
  .modal-backdrop {
    position: fixed;
    inset: 0;
    background: rgba(0, 0, 0, 0.6);
    display: flex;
    align-items: center;
    justify-content: center;
    z-index: 100;
    backdrop-filter: blur(4px);
  }
  .modal {
    background: rgba(20, 10, 35, 0.97);
    border: 1px solid var(--color-border);
    border-radius: 1rem;
    box-shadow: 0 8px 32px rgba(0, 0, 0, 0.7);
    min-width: 320px;
    max-width: 420px;
    width: 90%;
    display: flex;
    flex-direction: column;
  }
  .modal-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 0.85rem 1.25rem;
    border-bottom: 1px solid var(--color-border);
    font-size: 0.9rem;
    font-weight: 500;
    color: var(--color-theme-2);
  }
  .modal-close {
    background: none;
    border: none;
    color: var(--color-subtle-text);
    cursor: pointer;
    font-size: 0.85rem;
    padding: 0;
    line-height: 1;
    transition: color 0.15s;
  }
  .modal-close:hover {
    color: #ff9090;
    background: none;
  }
  .modal-body {
    padding: 1.25rem;
    display: flex;
    flex-direction: column;
    gap: 1rem;
  }
  .file-preview {
    display: flex;
    align-items: center;
    justify-content: space-between;
    background: rgba(255, 255, 255, 0.04);
    border: 1px solid var(--color-border);
    border-radius: 0.5rem;
    padding: 0.65rem 0.85rem;
  }
  .file-preview-name {
    font-size: 0.85rem;
    color: var(--color-text);
    font-family: var(--font-mono);
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
    max-width: 70%;
  }
  .file-preview-size {
    font-size: 0.75rem;
    color: var(--color-subtle-text);
    font-family: var(--font-mono);
    flex-shrink: 0;
  }
  .field-row {
    display: flex;
    flex-direction: column;
    gap: 0.5rem;
  }
  .field-row label {
    font-size: 0.75rem;
    color: var(--color-subtle-text);
    text-transform: uppercase;
    letter-spacing: 0.08em;
  }
  .vis-options {
    display: flex;
    gap: 0.4rem;
  }
  .vis-btn {
    flex: 1;
    padding: 0.4rem 0.5rem;
    font-size: 0.8rem;
    background: rgba(255, 255, 255, 0.03);
    border: 1px solid var(--color-border);
    border-radius: 0.4rem;
    color: var(--color-subtle-text);
    cursor: pointer;
    transition: all 0.15s ease;
    font-family: var(--font-body);
  }
  .vis-btn:hover {
    background: rgba(255, 255, 255, 0.07);
    color: var(--color-text);
  }
  .vis-btn.active {
    background: rgba(126, 156, 255, 0.15);
    border-color: rgba(126, 156, 255, 0.4);
    color: var(--color-theme-1);
  }
  .vis-hint {
    font-size: 0.78rem;
    color: var(--color-subtle-text);
    margin: 0;
    line-height: 1.5;
  }
  .modal-footer {
    display: flex;
    justify-content: flex-end;
    gap: 0.5rem;
    padding: 0.85rem 1.25rem;
    border-top: 1px solid var(--color-border);
  }
  .btn-ghost {
    background: transparent;
    border: 1px solid var(--color-border);
    color: var(--color-subtle-text);
    padding: 0.45rem 1rem;
    border-radius: 0.4rem;
    font-size: 0.875rem;
    cursor: pointer;
    font-family: var(--font-body);
    transition: all 0.15s ease;
  }
  .btn-ghost:hover {
    background: rgba(255, 255, 255, 0.06);
    color: var(--color-text);
  }
  .btn-confirm {
    background: rgba(126, 156, 255, 0.2);
    border: 1px solid rgba(126, 156, 255, 0.4);
    color: var(--color-theme-1);
    padding: 0.45rem 1.25rem;
    border-radius: 0.4rem;
    font-size: 0.875rem;
    cursor: pointer;
    font-family: var(--font-body);
    transition: all 0.15s ease;
    font-weight: 500;
  }
  .btn-confirm:hover:not(:disabled) {
    background: rgba(126, 156, 255, 0.32);
  }
  .btn-confirm:disabled {
    opacity: 0.5;
    cursor: not-allowed;
  }

  @media (max-width: 600px) {
    .top-row {
      flex-direction: column;
      align-items: flex-start;
    }
    .btn-primary {
      width: 100%;
    }
    .grid {
      grid-template-columns: 1fr;
    }
    .modal-backdrop {
      align-items: flex-end;
    }
    .modal {
      max-width: 100%;
      width: 100%;
      border-radius: 1rem 1rem 0 0;
      max-height: 90vh;
      overflow-y: auto;
    }
  }

  .pills {
    display: flex;
    flex-wrap: wrap;
    align-items: center;
    gap: 0.4rem;
    flex: 1;
  }

  .pill {
    display: inline-flex;
    align-items: center;
    gap: 0.35rem;
    background: rgba(126, 156, 255, 0.12);
    border: 1px solid rgba(126, 156, 255, 0.25);
    border-radius: 2rem;
    padding: 0.2rem 0.6rem 0.2rem 0.75rem;
    font-size: 0.8rem;
    color: var(--color-theme-1);
  }

  .pill-remove {
    background: none;
    border: none;
    cursor: pointer;
    color: rgba(126, 156, 255, 0.5);
    padding: 0;
    display: flex;
    align-items: center;
    transition: color 0.15s;
  }

  .pill-remove:hover {
    color: #ff9090;
  }

  .add-btn {
    background: none;
    border: 1px solid var(--color-border);
    border-radius: 2rem;
    padding: 0.2rem 0.65rem;
    font-size: 0.78rem;
    color: var(--color-subtle-text);
    cursor: pointer;
    transition: all 0.15s;
    font-family: var(--font-body);
  }

  .add-btn:hover {
    background: rgba(255, 255, 255, 0.06);
    color: var(--color-text);
  }

  .search-wrap {
    border-bottom: 1px solid var(--color-border);
    display: flex;
    flex-direction: column;
  }

  .search-input {
    margin: 0.5rem 1rem;
    background: rgba(255, 255, 255, 0.05);
    border: 1px solid var(--color-border);
    border-radius: 0.4rem;
    padding: 0.45rem 0.75rem;
    color: var(--color-text);
    font-family: var(--font-body);
    font-size: 0.85rem;
    outline: none;
  }

  .search-input:focus {
    border-color: rgba(126, 156, 255, 0.4);
  }

  .search-status {
    padding: 0.75rem 1rem;
    font-size: 0.82rem;
    color: var(--color-subtle-text);
    text-align: center;
  }

  .search-result {
    display: flex;
    align-items: center;
    gap: 0.65rem;
    padding: 0.6rem 1rem;
    background: none;
    border: none;
    cursor: pointer;
    width: 100%;
    text-align: left;
    font-family: var(--font-body);
    transition: background 0.15s;
  }

  .search-result:hover {
    background: rgba(255, 255, 255, 0.04);
  }

  .user-avatar {
    width: 30px;
    height: 30px;
    border-radius: 50%;
    background: rgba(126, 156, 255, 0.15);
    border: 1px solid rgba(126, 156, 255, 0.2);
    display: flex;
    align-items: center;
    justify-content: center;
    font-size: 0.7rem;
    font-weight: 600;
    color: var(--color-theme-1);
    flex-shrink: 0;
  }

  .user-info {
    display: flex;
    flex-direction: column;
    gap: 0.1rem;
  }
  .user-name {
    font-size: 0.875rem;
    color: var(--color-text);
    font-weight: 500;
  }
  .user-bio {
    font-size: 0.75rem;
    color: var(--color-subtle-text);
  }

  .compose-title {
    flex: 1;
    background: none;
    border: none;
    outline: none;
    color: var(--color-text);
    font-family: var(--font-body);
    font-size: 0.9rem;
  }

  .compose-title::placeholder {
    color: var(--color-subtle-text);
    opacity: 0.5;
  }

  .send-btn {
    align-self: flex-end;
    margin: 0 1rem 1rem;
    background: rgba(126, 156, 255, 0.15);
    border: 1px solid rgba(126, 156, 255, 0.35);
    color: var(--color-theme-1);
    padding: 0.5rem 1.25rem;
    border-radius: 0.4rem;
    font-size: 0.875rem;
    cursor: pointer;
    font-family: var(--font-body);
    transition: all 0.2s;
  }
</style>
