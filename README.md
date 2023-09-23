# DSH CLI tool
[![Build Status](https://github.com/Arend-Jan/dsh/actions/workflows/main.yml/badge.svg)](https://github.com/Arend-Jan/dsh/actions/workflows/main.yml)
[![License](https://img.shields.io/badge/License-Apache%202.0-blue.svg)](https://opensource.org/licenses/Apache-2.0)

## Maturity
It is still very early days for this application. No real testing or looking at security implications has been conducted. Don't use this tool for any real project.

!!! secrets (like the API key) are not securly stored on OS, but in plain text. !!!

## General discription
A CLI application for the [`KPN Data Services Hub (DSH)`](https://kpn.com/dsh), featuring:
- Powerfull concurrent retreival of tokens
- Writing tokens to stdout
- Writing tokens to file
- Basic MQTT client

## Install
This tool is created in [`Rust`](https://www.rust-lang.org/). Therefor the prerequisite is to install Rust itself.

### Rust
```bash
# install rust
> curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

#### Configuring the PATH environment variable
In the Rust development environment, all tools are installed to the ~/.cargo/bin directory, and this is where you will find the Rust toolchain, including rustc, cargo, and rustup.

Accordingly, it is customary for Rust developers to include this directory in their PATH environment variable. During installation rustup will attempt to configure the PATH. Because of differences between platforms, command shells, and bugs in rustup, the modifications to PATH may not take effect until the console is restarted, or the user is logged out, or it may not succeed at all.

If, after installation, running rustc --version in the console fails, this is the most likely reason.

### OpenSSL
The application is depended on the availablity openssl-dev. The openssl-sys crate will automatically detect OpenSSL installations via Homebrew on macOS and vcpkg on Windows. Additionally, it will use pkg-config on Unix-like systems to find the system installation.

``` bash
> brew install openssl@1.1

> sudo port install openssl

> sudo pkgin install openssl

> sudo pacman -S pkg-config openssl

> sudo apt-get install pkg-config libssl-dev

> sudo dnf install pkg-config openssl-devel
```

### Installing the application
```bash
# install dsh_token_fetcher (must be in the root folder of this project)
> cargo install --path .
```

## How to use the CLI
```bash
# See the help flag for how to use this cli tool
> dsh --help
# 
# For the subcommands use
> dsh <subcommand> --help
```

## License
License: Apache License 2.0
