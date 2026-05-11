import { API_URL, createClient } from "$lib/utils/auth.svelte.ts";
import type { components, paths as GradesPaths } from "$lib/types/gradegetter.api";
export type GradeGetterHashMap = components["schemas"]["HashMap"]

export const gradesApi = createClient<GradesPaths>(`${API_URL}/gradegetter`);

let _grades = $state({});
let _forbiddon = $state(false);

export const grades = {
  get value() { return _grades },
  set value(v: GradeGetterHashMap) { _grades = v }
};

export const forbidden = {
  get value() { return _forbiddon },
  set value(v) { _forbiddon = v }
};

export async function fetchGrades() {

  const grades_req = gradesApi.path("/grades").method("get").create();
  const response = await grades_req({});

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

  grades.value = response.data;
}
