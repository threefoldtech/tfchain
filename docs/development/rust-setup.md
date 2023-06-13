---
title: Installation
---

This guide is for reference only, please check the latest information on getting started with Substrate
[here](https://docs.substrate.io/main-docs/install/).

This page will guide you through the **2 steps** needed to prepare a computer for **Substrate** development.
Since Substrate is built with [the Rust programming language](https://www.rust-lang.org/), the first
thing you will need to do is prepare the computer for Rust development - these steps will vary based
on the computer's operating system. Once Rust is configured, you will use its toolchains to interact
with Rust projects; the commands for Rust's toolchains will be the same for all supported,
Unix-based operating systems.

## Build dependencies

Substrate development is easiest on Unix-based operating systems like macOS or Linux. The examples
in the [Substrate Docs](https://docs.substrate.io) use Unix-style terminals to demonstrate how to
interact with Substrate from the command line.

### Ubuntu/Debian

Use a terminal shell to execute the following commands:

```sh
sudo apt update
# May prompt for location information
sudo apt install -y git clang curl libssl-dev llvm libudev-dev
```

### Arch Linux

Run these commands from a terminal:

```sh
pacman -Syu --needed --noconfirm curl git clang
```

### Fedora

Run these commands from a terminal:

```sh
sudo dnf update
sudo dnf install clang curl git openssl-devel
```

### OpenSUSE

Run these commands from a terminal:

```sh
sudo zypper install clang curl git openssl-devel llvm-devel libudev-devel
```

### macOS

> **Apple M1 ARM**
> If you have an Apple M1 ARM system on a chip, make sure that you have Apple Rosetta 2
> installed through `softwareupdate --install-rosetta`. This is only needed to run the
> `protoc` tool during the build. The build itself and the target binaries would remain native.

Open the Terminal application and execute the following commands:

```sh
# Install Homebrew if necessary https://brew.sh/
/bin/bash -c "$(curl -fsSL https://raw.githubusercontent.com/Homebrew/install/master/install.sh)"

# Make sure Homebrew is up-to-date, install openssl
brew update
brew install openssl
```

### Windows

**_PLEASE NOTE:_** Native Windows development of Substrate is _not_ very well supported! It is _highly_
recommend to use [Windows Subsystem Linux](https://docs.microsoft.com/en-us/windows/wsl/install-win10)
(WSL) and follow the instructions for [Ubuntu/Debian](#ubuntudebian).
Please refer to the separate
[guide for native Windows development](https://docs.substrate.io/main-docs/install/windows/).

## Rust developer environment

This guide uses <https://rustup.rs> installer and the `rustup` tool to manage the Rust toolchain.
First install and configure `rustup`:

```sh
# Install
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
# Configure
source ~/.cargo/env
```

Configure the Rust toolchain to default to the latest stable version:

```sh
rustup default stable
rustup update
```

## Test your set-up

Now the best way to ensure that you have successfully prepared a computer for Substrate
development is to follow the steps in [our first Substrate tutorial](https://docs.substrate.io/tutorials/v3/create-your-first-substrate-chain/).

## Troubleshooting Substrate builds

Sometimes you can't get the Substrate node template
to compile out of the box. Here are some tips to help you work through that.

### Rust configuration check

To see what Rust toolchain you are presently using, run:

```sh
rustup show
```

This will show something like this (Ubuntu example) output:

```text
Default host: x86_64-unknown-linux-gnu
rustup home:  /home/user/.rustup

installed toolchains
--------------------

stable-x86_64-unknown-linux-gnu (default)
nightly-2020-10-06-x86_64-unknown-linux-gnu
nightly-x86_64-unknown-linux-gnu

installed targets for active toolchain
--------------------------------------

wasm32-unknown-unknown
x86_64-unknown-linux-gnu

active toolchain
----------------

stable-x86_64-unknown-linux-gnu (default)
rustc 1.70.0 (90c541806 2023-05-31)
```

You can see that the default toolchain is `stable-x86_64-unknown-linux-gnu` and that the `wasm32-unknown-unknown` target is installed.
