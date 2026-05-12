import { goto } from "$app/navigation";
import { Fetcher } from "openapi-typescript-fetch";
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


export const API_URL = "https://api.devinlittle.net";
//export const API_URL = "https://localhost:8082";
//export const API_URL = import.meta.env.API_URL;

// PROD KEY
export const VAPID_PUBLIC_KEY = "BNw9vPwkS56SfucmmgVchwetJmkrVjUcyPf6mv3J0Wl7il4b3Y_OmVgEfbC4RWUoxYZPq1eZYqah8CtjLQm_7uU"
// TESTING KEY
//export const VAPID_PUBLIC_KEY = "BDo9oD-LKEaTbSP4-arFhXUBl2OkV0InKjMFnLVxb3xTcNejP0mmLFOh_nZ_s-4Q5sW_FEjy8maH03jBDclxIH4"

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

export function createClient<T>(baseUrl: string) {
  const fetcher = Fetcher.for<T>();

  fetcher.configure({
    baseUrl,
    init: { credentials: "include" },
    use: [
      async (url, init, next) => {
        const headers = new Headers(init.headers);

        if (auth.accessToken) {
          headers.set("Authorization", `Bearer ${auth.accessToken}`);
        }

        try {
          let res = await next(url, { ...init, headers });

          if (res.status === 401 || res.status === 403) {
            return await preformRefresh(url, init, next);
          }

          return res;
        } catch (err) {
          if (err.status === 401 || err.status === 403) {
            return await preformRefresh(url, init, next);
          }
          throw err;
        }
      }
    ]
  });

  return fetcher;
}

let refreshPromise: Promise<boolean> | null = null;

async function getRefreshStatus() {
  if (refreshPromise) {
    return refreshPromise;
  }

  refreshPromise = refresh();

  try {
    return await refreshPromise;
  } finally {
    refreshPromise = null;
  }
}

async function preformRefresh(url: any, init: any, next: any) {
  const ok = await getRefreshStatus();
  if (ok) {
    const retryHeaders = new Headers(init.headers);
    retryHeaders.set("Authorization", `Bearer ${auth.accessToken}`);

    return await next(url, { ...init, headers: retryHeaders });
  }
  throw new Error("Refresh failed");
}

import type { paths as AuthPaths, components } from "$lib/types/auth.api";
export const authApi = createClient<AuthPaths>(`${API_URL}/auth`);

export type ServiceName = components["schemas"]["ServiceName"]
export type UserRole = components["schemas"]["UserRole"]

function decode(token) {
  try {
    return JSON.parse(atob(token.split(".")[1])); // grabs the data portion of the jwt, ignoring header and sig
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
  let get_sessions = authApi.path("/me/sessions").method("get").create();

  const sessions_req = await get_sessions({});
  if (sessions_req.ok) {
    let sessions = sessions_req.data;
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

export function getRole(roles: UserRole, service: ServiceName): UserRole {
  return roles[service] ?? roles["global"] ?? "user";
}

// INFO: called once in +layout.svelte onMount
export async function initAuth() {
  try {
    const refresh = authApi.path("/refresh").method("post").create();
    const res = await refresh({});

    if (res.ok) {
      const { access_token } = res.data;
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


export async function logout() {
  const logout = authApi.path("/logout").method("get").create();
  await logout({});
  clear();
  goto("/");
}

// INFO: called from login/register page after successful auth
export function onAuthSuccess(token) {
  setToken(token);
}


// INFO: USE CREATE CLIENT INSTEAD

/*export async function _authFetch(input, init = {}) {
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
*/
