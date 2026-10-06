import { defineEnvVars } from '@sveltejs/kit/env';

export const variables = defineEnvVars({
  PUBLIC_TURN_USERNAME: { public: true, static: true },
  PUBLIC_TURN_PASSWORD: { public: true, static: true }
});
