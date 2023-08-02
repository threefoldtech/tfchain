# Setup Your Development Environment

## Installing Required Packages and Rust

Before you can start developing a Substrate-based blockchain, you need to prepare your development environment with the required packages and Rust.
For installation instructions, see the appropriate topic for your operating system.

- [Linux](https://docs.substrate.io/install/linux/)
- [macOS](https://docs.substrate.io/install/macos/)
- [Windows](https://docs.substrate.io/install/windows/)

# VSCode configuration:

VSCode rust analyzer is a VS Code extension that provides support for the Rust programming language. It offers features such as code completion, go to definition, find references, syntax highlighting, and more. To install it, you need to follow these steps:
- you need to open VS Code and go to the Extensions view by pressing Ctrl+Shift+X or clicking on the Extensions icon on the left sidebar.
- Then, you need to search for “rust-analyzer” in the search box and click on the Install button next to the extension with the name “rust-analyzer” and the publisher “The Rust Programming Language”.
- Finally, you need to reload VS Code to activate the extension. You can do that by clicking on the Reload button that appears after the installation is complete.
That’s it! You have successfully installed vscode rust analyzer and you can start using it to write and edit Rust code.

# Subkey:

`subkey` is a key generation and management utility Used to generate public and private key pairs for users or validators, and do a lot more

For the installation and basic usage instructions of `subkey` standalone program, see [here](https://docs.substrate.io/reference/command-line-tools/subkey/).

Also you can use the node-template [key](https://docs.substrate.io/reference/command-line-tools/node-template/#key) command to generate, inspect, and manage private and public key pairs and addresses. The node-template `key` command provides convenient access to a subset of key management services that are available in the standalone program and most of the node-template key subcommands are identical to subkey subcommands.

