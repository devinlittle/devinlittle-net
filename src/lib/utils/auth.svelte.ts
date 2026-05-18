import { goto } from "$app/navigation";
import { Fetcher } from "openapi-typescript-fetch";
import { API_URL } from "./constants.svelte";
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

export const global_private_key = {
  get value() { return _global_private_key },
  set value(v: CryptoKey) { _global_private_key = v }
};

let _global_private_key = $state<CryptoKey>();

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
            try {
              await preformRefresh(url, init, next);
            } catch (err) {
              return res;
            }
          }

          return res;
        } catch (err) {
          if (err.status === 401 || err.status === 403) {
            try {
              await preformRefresh(url, init, next);
            } catch (err_not_used) {
              return err;
            }
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
  return new Error("Refresh failed")
  //throw new Error("Refresh failed");
}

import type { paths as AuthPaths, components } from "$lib/types/auth.api";
import { base64_to_arraybuffer } from "./smalltalk.svelte";
import { db_state, get_private_key_from_indexeddb, mountDB } from "./sqlite.svelte";
import { addNotification, connectNotifications, disconnectNotifications } from "./notifications.svelte";
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

async function setToken(token) {
  auth.accessToken = token;
  auth.id = decode(token)?.sub ?? null;
  auth.username = decode(token)?.username ?? null;
  auth.roles = decode(token)?.roles ?? null;

  localStorage.setItem("access_token", token);

  if (decode(token)?.public_key) {
    try {
      auth.public_key = await return_public_key(decode(token)?.public_key);
    } catch (e) {
      console.error("Failed to import public key:", e);
      auth.public_key = null;
    }
  } else {
    auth.public_key = null;
  }

  try {
    global_private_key.value = await get_private_key_from_indexeddb();
  } catch (e) {
    console.error("error adding private_key to auth");
    global_private_key.value = null;
  }

  auth.ready = true;
}

async function return_public_key(public_key_b64: string): Promise<CryptoKey> {
  if (public_key_b64 == null) {
    auth.public_key = null
    return;
  }

  let public_key_buf = base64_to_arraybuffer(public_key_b64);

  let public_key = await crypto.subtle.importKey(
    'spki',
    public_key_buf,
    { name: 'ECDH', namedCurve: 'P-256' },
    true,
    []
  );

  return public_key;
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
  auth.public_key = null;
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
      await setToken(access_token);
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
  await setToken(access_token);
  await setSessionId();
  return true;
}


export async function logout() {
  const logout = authApi.path("/logout").method("get").create();
  await logout({});
  clear();
  goto("/");
  disconnectNotifications();
  connectNotifications();
}

// INFO: called from login/register page after successful auth
export async function onAuthSuccess(token) {
  await setToken(token);
  await setSessionId();
}


export async function get_ready_for_devin_grfd(from_login: boolean): Promise<boolean> {
  disconnectNotifications();
  if (!from_login) {
    const ok = await initAuth();
    if (!ok) {
      connectNotifications();
      return false;
    }
  }
  if (auth.ready) {
    console.log("pretend the db is being setup");
    await mountDB();
    if (db_state.ready) {
    } else if (db_state.needs_key_sync) {
      addNotification({
        type: "important_info",
        title: "Notification",
        body: "messaging capabilities are limited, navigate to the settings page to sync encryption keys",
        sender: "DevinLittle.Net",
        global: false,
      });
    } else if (db_state.needs_onboarding) {
      // generate keys
    }
  }

  console.log("begining notification connectoin")
  connectNotifications();
  console.log("done doing that")
  return true;
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
