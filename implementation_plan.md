# Build FocusOS Setup Executable and Release Page

This plan outlines how we will build the `FocusOS_Setup.exe` installer for your app and set up a streamlined GitHub Releases page for users to download it.

## Open Questions

> [!NOTE]
> Do you want to build the `.exe` file locally on your computer and upload it manually to GitHub, or would you prefer a **GitHub Actions workflow** that automatically builds and creates a release whenever you push a new version tag (e.g., `v0.1.0`)? (Automated is highly recommended for open-source projects).

## Proposed Changes

### Option A: Local Build (Manual Upload)
If you want to build it right now on your machine:

1. **Install Prerequisites**: Ensure you have Node.js and Rust installed (which you already do).
2. **Build the App**: We will run the Tauri build command.
   ```bash
   cd apps/desktop
   npm run tauri build
   ```
3. **Locate the Installer**: Once completed, the setup `.exe` will be located at:
   `apps/desktop/src-tauri/target/release/bundle/nsis/FocusOS_0.1.0_x64-setup.exe`
4. **Rename and Upload**: You can rename this file to `FocusOS_Setup.exe`, go to your GitHub repository's Releases page, draft a new release, and attach the file.

### Option B: Automated Build via GitHub Actions (Recommended)
This approach sets up a script so GitHub servers build the `.exe` for you and automatically attach it to a release.

#### [NEW] .github/workflows/release.yml
We will create a GitHub Actions file that does the following:
- Triggers whenever you push a tag like `v*` (e.g., `v0.1.0`).
- Sets up Node.js and Rust on a Windows GitHub runner.
- Installs dependencies and runs `npm run tauri build`.
- Automatically renames the generated setup file to `FocusOS_Setup.exe`.
- Publishes it to the GitHub Releases page with your release notes.

### Option C: Complete Docker Setup
If you want to containerize the development and build environment for this project, we can create a complete Docker setup. This is especially useful for ensuring consistent builds across different machines or for building Linux/Web versions of your app without installing Rust/Node natively.

#### [NEW] Dockerfile
We will create a multi-stage `Dockerfile`:
- **Stage 1 (Frontend Dev)**: Sets up Node.js and dependencies to run the Vite development server (useful if you want to test the React UI in a browser).
- **Stage 2 (Tauri Builder)**: Sets up Rust, necessary system dependencies (like `libwebkit2gtk` for Linux builds or cross-compilation tools for Windows), and builds the final executable.

#### [NEW] docker-compose.yml
We will create a `docker-compose.yml` to orchestrate your workflow:
- A `dev` service to run the Vite frontend in a container with hot-reloading.
- A `build-windows` service (optional, if using Wine/cross-compilation) or `build-linux` service to output the generated executables to a local `/dist` volume.

## Verification Plan

### Manual Verification
- Review the generated `.exe` to ensure it successfully installs FocusOS.
- Verify the GitHub Releases page correctly links to the `FocusOS_Setup.exe` file, matching your README instructions.
- If Option C (Docker) is chosen, verify that `docker compose up dev` starts the frontend and `docker compose run build` successfully compiles the app.

---
**Please review and let me know which combination of options (e.g., Option B + Option C) you prefer, and we will proceed!**
