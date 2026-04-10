<script>
  import { goto } from "$app/navigation";

  import { API_URL, onAuthSuccess } from "$lib/utils/auth.svelte.js";

  let tab = $state("login");

  let loginUsername = $state("");
  let loginPassword = $state("");
  let loginError = $state("");

  let regUsername = $state("");
  let regPassword = $state("");
  let regConfirm = $state("");
  let regError = $state("");

  async function handleLogin() {
    loginError = "";
    if (!loginUsername || !loginPassword) {
      loginError = "fill in all fields";
      return;
    }

    const res = await fetch(`${API_URL}/auth/login`, {
      method: "POST",
      headers: { "Content-Type": "application/json" },
      credentials: "include",
      body: JSON.stringify({
        username: loginUsername,
        password: loginPassword,
      }),
    });

    if (!res.ok) {
      loginError =
        res.status === 401
          ? "invalid username or password"
          : "something went wrong";
      return;
    }

    const { access_token } = await res.json();
    onAuthSuccess(access_token);
    goto("/");
  }

  async function handleRegister() {
    regError = "";
    if (!regUsername || !regPassword || !regConfirm) {
      regError = "fill in all fields";
      return;
    }
    if (regPassword !== regConfirm) {
      regError = "passwords do not match";
      return;
    }

    const res = await fetch(`${API_URL}/auth/register`, {
      method: "POST",
      headers: { "Content-Type": "application/json" },
      credentials: "include",
      body: JSON.stringify({ username: regUsername, password: regPassword }),
    });

    if (!res.ok) {
      regError =
        res.status === 409 ? "username already taken" : "something went wrong";
      return;
    }

    const login_res = await fetch(`${API_URL}/auth/login`, {
      method: "POST",
      headers: { "Content-Type": "application/json" },
      credentials: "include",
      body: JSON.stringify({ username: regUsername, password: regPassword }),
    });

    const { access_token } = await login_res.json();

    localStorage.setItem("access_token", access_token);
    goto("/");
  }
</script>

<div class="wrapper">
  <div class="tabs">
    <button class:active={tab === "login"} onclick={() => (tab = "login")}
      >login</button
    >
    <button class:active={tab === "register"} onclick={() => (tab = "register")}
      >register</button
    >
  </div>

  {#if tab === "login"}
    <div class="form">
      <input type="text" placeholder="username" bind:value={loginUsername} />
      <input
        type="password"
        placeholder="password"
        bind:value={loginPassword}
        onkeydown={(e) => e.key === "Enter" && handleLogin()}
      />
      {#if loginError}<p class="error">{loginError}</p>{/if}
      <button onclick={handleLogin}>Sign in</button>
    </div>
  {:else}
    <div class="form">
      <input type="text" placeholder="username" bind:value={regUsername} />
      <input type="password" placeholder="password" bind:value={regPassword} />
      <input
        type="password"
        placeholder="confirm password"
        bind:value={regConfirm}
        onkeydown={(e) => e.key === "Enter" && handleRegister()}
      />
      {#if regError}<p class="error">{regError}</p>{/if}
      <button onclick={handleRegister}>Create account</button>
    </div>
  {/if}
</div>

<style>
  .wrapper {
    max-width: 360px;
    margin: var(--column-margin-top) auto;
    padding: 2rem;
    background: rgba(255, 255, 255, 0.03);
    border: 1px solid var(--color-border);
    border-radius: 1rem;
  }

  .tabs {
    display: flex;
    gap: 0.25rem;
    background: rgba(255, 255, 255, 0.04);
    border-radius: 0.5rem;
    padding: 0.25rem;
    margin-bottom: 1.5rem;
  }

  .tabs button {
    flex: 1;
    padding: 0.4rem 0;
    background: none;
    border: none;
    border-radius: 0.35rem;
    cursor: pointer;
    color: inherit;
    opacity: 0.5;
    transition:
      background 0.15s,
      opacity 0.15s;
  }

  .tabs button.active {
    background: rgba(255, 255, 255, 0.08);
    opacity: 1;
  }

  .form button {
    width: 100%;
    padding: 0.5rem;
    cursor: pointer;
  }

  .form input {
    width: 100%;
    box-sizing: border-box;
  }

  .form {
    display: flex;
    flex-direction: column;
    gap: 0.75rem;
  }

  .form input {
    width: 100%;
  }

  .error {
    font-size: 0.8rem;
    color: #ff7eb3;
    margin: 0;
  }
</style>
