# General Information:
  * If you are on windows its recommended to use WSL rather than a normal Windows host
  * Docker images _will_ eventually provided with all mentioned prerequisites pre-installed; so what I'm getting at is if you don't want to install all these and rather use a docker image to build, that option is available to you.

## General Prerequisites
  * [`rustup`](https://rustup.rs)
    * this includes cargo as cargo is used heavily
  * [`bun`](https://bun.sh)
    * bun is used solely as a package manager in this project as its fast and great
  * `node`
    * I recommend setting up node with [fnm](https://github.com/Schniz/fnm), node is used as a fallback here for when bun doesn't have compatibility
  * [`just`](https://github.com/casey/just)
    * just is used as the modern make file and the task runner I chose for this project

# Backend Services

Change Directories into the [services](./services) directory to run Just tasks anddd yeah thats kinda it, run a `just help` to see all possible tasks to run and one of them is the correct building one.

# Building the Apps

Packaging the apps is really annoying as I'm trying to support every platform that makes reasonable sense on 2 architectures so just comes in clutch here.

## Prerequisites
  * `cargo-zigbuild`
    * Use cargo-zigbuild when targeting macOS on a darwin (OSX) device
  * `cargo-xwin`
    * windows users are expected to build using WSL so its expected to use install cargo-xwin to handle the CRT and SDK stuff for u
  * `nfpm`
    * the packer I chose to create both DEBs and RPMs
