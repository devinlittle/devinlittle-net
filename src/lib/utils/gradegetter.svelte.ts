import { API_URL, authFetch } from "$lib/utils/auth.svelte.ts";

let _grades = $state({});
let _forbiddon = $state(false);

export const grades = {
  get value() { return _grades },
  set value(v) { _grades = v }
};

export const forbidden = {
  get value() { return _forbiddon },
  set value(v) { _forbiddon = v }
};

export async function fetchGrades() {
  const response = await authFetch(`${API_URL}/gradegetter/grades`, {
    method: "GET",
    headers: {
      "Content-Type": "application/json",
    },
  });

  if (!response.ok) {

    forbidden.value = true
    if (response.status === 403) {
      // Response is Forbiddon
      forbidden.value = true;
      return;
    }
    console.error("Failed to fetch grades");
    return;
  }

  forbidden.value = false;

  grades.value = await response.json();
}
