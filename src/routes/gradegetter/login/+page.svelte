<script>
  import { goto } from "$app/navigation";

  async function login(event) {
    event.preventDefault();
    const form = new FormData(event.target);
    const username = form.get("username");
    const password = form.get("password");
    let apiUrl = "api.devinlittle.net";

    const response = await fetch(`https://${apiUrl}:3000/auth/login`, {
      method: "POST",
      headers: { "Content-Type": "application/json" },
      body: JSON.stringify({ username, password }),
    });

    if (!response.ok) {
      console.error("Login failed");
      return;
    }

    let token = await response.json();

    localStorage.setItem("token", token);

    goto("/gradegetter");
  }
</script>

<h1>Login!</h1>

<form onsubmit={login}>
  <label>
    Username
    <input name="username" type="text" required />
  </label>
  <label>
    Password
    <input name="password" type="password" required />
  </label>
  <button type="submit">Login</button>
</form>
