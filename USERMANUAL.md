# Prerequisites

You need to have project’s source code and the the following tools installed:

- Rust (https://www.rust-lang.org/tools/install)
- Node.js (https://nodejs.org/en/download/package-manager)
  Navigate to the server folder located at the root of the project's source code.
  Then install the Node.js packages for the front-end by running the command npm install.

# Compiling

To begin, navigate to the root folder of the project and access the server folder. Launch a command prompt there. For compiling both the client and the server, use the command `cargo build`, which by default builds the source code in debug mode. To compile in release mode, add the `--release` parameter to the build command.  
Since we are using a cargo workspace, use the `-p` parameter (short for `--package`) to specify whether to compile the client or the server.  
Compile the client and the server in debug mode with the following commands: `cargo build -p client` and `cargo build -p server`. The compiled binaries will be located under the target folder, which contains both release and debug folders.

# Releasing for production

To release the code for production, consider the following steps. Compiling the server in release mode is not recommended for a Tauri project, as it's simpler to create a Windows installer without extra folders from the compiling process. Instead of removing unnecessary files and folders from the release folder, it's more efficient to use the built tools directly.  
Navigate to the server directory, open a command prompt, and run `npm run tauri build`. This command should produce two bundles: an MSI installer and an NSIS installer. You may use either.  
The final step is to copy the pre-built client binary from before into the root folder of the server installation location. This step is necessary for the client builder feature in the server.

# Working on the code

If you prefer to skip the compilation process and run both the server and client projects immediately, use the `cargo run` command with the same parameters as the build command. For the server project, since the front-end is a Node.js project, run it in development mode for hot reloading (useful if making changes to the front-end) by navigating to the server folder and executing `npm run dev`.
