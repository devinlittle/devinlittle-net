#!/usr/bin/env bash
services=("auth" "gradegetter" "nanopass" "notification", "smalltalk")

for s in "${services[@]}"; do
  bunx openapi-typescript "https://api.devinlittle.net/$s/api-docs/openapi.json" -o "./src/lib/types/$s.api.ts"
done
