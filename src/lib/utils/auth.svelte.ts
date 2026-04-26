import { goto } from "$app/navigation";
/* export const fetchMarkdownPosts = async () => {
  const allPostFiles = import.meta.glob('/src/routes/projects/*.md');
  const iterablePostFiles = Object.entries(allPostFiles);

  const allPosts = await Promise.all(
    iterablePostFiles.map(async ([path, resolver]) => {
      const { metadata } = await resolver();
      const postPath = path.slice(11, -3);

      return {
        meta: metadata,
        path: postPath
      };
    })
  );

  return allPosts ;
}; */


//export const API_URL = "https://api.devinlittle.net";
export const API_URL = "https://localhost:8082";
//export const API_URL = import.meta.env.API_URL;

// this "auth" var is for state
export let auth = $state({
  id: null,
  session_id: null,
  username: null,
  roles: null,
  public_key: null,
  accessToken: null,
  ready: false
});

function decode(token) {
  try {
    return JSON.parse(atob(token.split(".")[1]));
  } catch {
    return null;
  }
}

function setToken(token) {
  auth.accessToken = token;
  auth.id = decode(token)?.sub ?? null;
  auth.username = decode(token)?.username ?? null;
  auth.roles = decode(token)?.roles ?? null;
  auth.public_key = decode(token)?.public_key ?? null;
  localStorage.setItem("access_token", token);
  auth.ready = true;
}

async function setSessionId() {
  const sessions_req = await authFetch(`${API_URL}/auth/me/sessions`);
  if (sessions_req.ok) {
    let sessions = await sessions_req.json()
    for (const session of sessions) {
      if (session.is_current) {
        auth.session_id = session.session_id;
      }
    }
  }
}

function clear() {
  auth.accessToken = null;
  auth.username = null;
  auth.id = null;
  auth.roles = null;
  auth.ready = false;
  localStorage.removeItem("access_token");
}

export function getRole(roles, service) {
  return roles[service] ?? roles["global"] ?? "user";
}

// INFO: called once in +layout.svelte onMount
export async function initAuth() {
  try {
    const res = await fetch(`${API_URL}/auth/refresh`, {
      method: "POST",
      credentials: "include",
    });
    if (res.ok) {
      const { access_token } = await res.json();
      setToken(access_token);
      await setSessionId()
      return true;
    }
  } catch {
    /* server down */
    return false;
  }
}

export async function refresh() {
  const res = await fetch(`${API_URL}/auth/refresh`, {
    method: "POST",
    credentials: "include",
  });
  if (!res.ok) { clear(); return false; }
  const { access_token } = await res.json();
  setToken(access_token);
  await setSessionId();
  return true;
}

export async function authFetch(input, init = {}) {
  const headers = new Headers(init.headers ?? {});
  if (auth.accessToken) headers.set("Authorization", `Bearer ${auth.accessToken}`);

  let res = await fetch(input, { ...init, headers, credentials: "include" });

  if (res.status === 401 || res.status === 403) {
    const ok = await refresh();
    if (ok) {
      headers.set("Authorization", `Bearer ${auth.accessToken}`);
      res = await fetch(input, { ...init, headers, credentials: "include" });
    }
  }
  return res;
}

export async function logout() {
  await fetch(`${API_URL}/auth/logout`, {
    method: "GET",
    credentials: "include",
  }).catch(() => { });
  clear();
  goto("/");
}

// INFO: called from login/register page after successful auth
export function onAuthSuccess(token) {
  setToken(token);
}
