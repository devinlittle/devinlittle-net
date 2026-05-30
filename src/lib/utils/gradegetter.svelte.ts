import { createClient } from "$lib/utils/auth.svelte";
import type { components, paths as GradesPaths } from "$lib/types/gradegetter.api";
import { API_URL } from "./constants.svelte";
export type GradeGetterHashMap = components["schemas"]["BTreeMap"]

export const gradesApi = createClient<GradesPaths>(`${API_URL}/gradegetter`);

// INFO: null here means everything is good
export type BadStatus = "no_gg_account" | "forbidden" | "not_found" | "internal_server_error" | null;

let _grades = $state({});
let _the_bad_status = $state<BadStatus>();

export const grades = {
  get value() { return _grades },
  set value(v: GradeGetterHashMap) { _grades = v }
};

export const the_bad_status = {
  get value() { return _the_bad_status },
  set value(v) { _the_bad_status = v }
};

export async function fetchGrades() {
  try {
    const grades_req = gradesApi.path("/grades").method("get").create();
    const response = await grades_req({});

    the_bad_status.value = null;
    grades.value = response.data;
  } catch (err) {
    switch (err.status) {
      case 401:
        the_bad_status.value = "no_gg_account"
        console.error("Failed to fetch grades");
        return;
      case 403:
        the_bad_status.value = "forbidden"
        console.error("Failed to fetch grades");
        return;
      case 404:
        the_bad_status.value = "not_found"
        console.error("Failed to fetch grades");
        return;
      case 500:
        the_bad_status.value = "internal_server_error"
        console.error("Failed to fetch grades");
        return;
    }
  }
}
