# My site

**This website is in progress**

To build, have pnpm installed and run

```sh
rm -rf ./build/ # only run if you have built the site before
pnpm install --frozen-lockfile # installs only the dev deps
pnpm build # builds the site
pnpm run start:prod # runs index.js in the build folder with node using the template.env file for environment variables 
```

* The site will be in the build. Run `node build` to run the site on `0.0.0.0` (localhost) with port `8000`.\
~~To change this you can add an env file more info [here](https://kit.svelte.dev/docs/adapter-node)~~\
Since node version 20.6.0 the nodejs team has added support for env files without the dotenv npm package more info [here](https://nodejs.org/en/blog/release/v20.6.0)*
