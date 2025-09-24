<script>
  import { goto } from "$app/navigation";
  let apiUrl = "api.devinlittle.net";

  let username = $state("");
  let password = $state("");
  let schoology_email = $state("");
  let schoology_password = $state("");
  let error = $state("");

  async function handleRegister(event) {
    error = "";

    try {
      // 1. Register the user
      const regRes = await fetch(`https://${apiUrl}:3000/auth/register`, {
        method: "POST",
        headers: { "Content-Type": "application/json" },
        body: JSON.stringify({ username, password }),
      });

      if (!regRes.ok) {
        const msg = await regRes.json();
        throw new Error(`Register failed: ${msg}`);
      }

      // 2. Login to get the token
      const loginRes = await fetch(`https://${apiUrl}:3000/auth/login`, {
        method: "POST",
        headers: { "Content-Type": "application/json" },
        body: JSON.stringify({ username, password }),
      });

      if (!loginRes.ok) {
        const msg = await loginRes.text();
        throw new Error(`Login failed: ${msg}`);
      }

      let token = await loginRes.json();
      localStorage.setItem("token", token);

      // 3. Register Schoology info
      const schoologyRes = await fetch(
        `https://${apiUrl}:3000/auth/schoology/credentials`,
        {
          method: "POST",
          headers: {
            "Content-Type": "application/json",
          },
          body: JSON.stringify({
            token,
            schoology_email,
            schoology_password,
          }),
        },
      );

      if (!schoologyRes.ok) {
        const msg = await schoologyRes.text();
        error = `${token}`;
        throw new Error(`Schoology registration failed: ${msg}`);
      }

      // 4. Run forward req
      const forwardRes = await fetch(`https://${apiUrl}:3000/auth/forward`, {
        method: "POST",
        headers: {
          "Content-Type": "application/json",
        },
        body: JSON.stringify({
          token,
        }),
      });

      if (!forwardRes.ok) {
        // const msg = await forwardRes.text();
      }

      // 4. Redirect to Root
      goto("/gradegetter");
    } catch (err) {
      error = err.message;
      console.error(err);
    }
  }
</script>

<form onsubmit={handleRegister}>
  <h1>Register!</h1>

  <label>
    Username:
    <input bind:value={username} required />
  </label>

  <label>
    Password:
    <input type="password" bind:value={password} required />
  </label>

  <h3>Schoology Info</h3>

  <label>
    Schoology Email:
    <input bind:value={schoology_email} required />
  </label>

  <label>
    Schoology Password:
    <input type="password" bind:value={schoology_password} required />
  </label>

  <button type="submit">Register</button>

  {#if error}
    <p style="color: red">{error}</p>
  {/if}
</form>
