# Project Context: The Vision (Project Resonate)

This file serves as a living document to track the context, architecture, purpose, and current state of the project. It will be updated continuously as the project evolves.

## 1. What is this project?
It is a **native, open-source music platform** that combines high-quality music streaming with a powerful local music library, giving users one place to discover, stream, organize, and listen to their music. It is designed to bring the convenience of a modern streaming service together with the ownership and control of a traditional music library, while maintaining a clean monochrome interface and a strong focus on audio quality. Rather than being built around ads, engagement loops, or locking the user's music behind a subscription, the goal is to create a platform where **streaming and personal ownership can coexist**—where music remains the focus, not the platform.

## 2. What does it stand for? (Vision & Goals)
It stands for **simplicity, ownership, and respect for music**—a native, open-source music experience built around the idea that listening to music should feel personal rather than commercial or distracting. It puts the listener and the music first, focusing on clean design, high-quality audio, local control, and freedom from unnecessary algorithms, ads, subscriptions, and engagement-driven features. It’s meant to feel less like a platform trying to keep you online and more like a piece of software that simply lets you **own your library, press play, and enjoy the music**.

## 3. Native Application Architecture
Resonate is built as a highly performant desktop application for Windows, utilizing **Tauri v2**. 

### How Tauri Works Under the Hood
1. **The Native WebView**: Instead of bundling a massive Chromium browser (like Electron), Tauri utilizes the OS's native web engine. On Windows, this is **WebView2** (Edge/Chromium based). The SvelteKit frontend is rendered directly inside this lightweight native window.
2. **Rust Backend**: The core processing is written in **Rust** (found in `src-tauri/`). Rust compiles directly to native Windows machine code (`.exe`). The Rust backend is responsible for all heavy lifting: local file system access (reading FLAC/MP3 files), complex audio processing, and network connections to Subsonic servers.
3. **Inter-Process Communication (IPC)**: The SvelteKit frontend communicates with the Rust backend via a secure IPC bridge. When a user interacts with the UI (e.g., clicking play), Svelte sends a command to Rust, which then executes the native system calls.
4. **The Build Process**: Running `npm run tauri build` builds the SvelteKit frontend to static files, compiles the Rust code, embeds the frontend, and packages the result into a standard, lightweight Windows installer (`.msi`).

## 4. Tech Stack
**Frontend (Web UI inside WebView2):**
- **Framework:** SvelteKit (Svelte 4) configured with `adapter-static` for purely client-side rendering (`ssr: false`).
- **Language:** TypeScript
- **Styling:** Vanilla CSS with a strict CSS Custom Properties design system (`tokens.css`). Tailwind is explicitly excluded.
- **Icons:** `lucide-svelte`
- **Typography:** Self-hosted local fonts (Geist, Inter, JetBrains Mono) for complete offline capability. No external CDNs.
- **Build Tool:** Vite

**Backend (Desktop Integration):**
- **Framework:** Tauri v2
- **Language:** Rust
- **Integration:** `@tauri-apps/api` and `@tauri-apps/plugin-shell` for native OS capabilities.

## 5. User Interface & Design System
The UI was entirely rebuilt to match a strictly defined "Material 3–flavored" dark-mode specification. 

### Design Principles
- **Layout:** A persistent left sidebar (`~256px`), a main content area for SvelteKit routing (`/albums`, `/tracks`, `/settings`), and a persistent bottom now-playing bar (`~72px`).
- **Aesthetic:** Dark-mode only for v1, sharp/near-flat corners (`2px` border radius on most elements, `12px` pill radius for small buttons/badges), and a highly refined typographic hierarchy.
- **Tokens:** All colors, spacing, and typography are defined in `src/lib/theme/tokens.css` (e.g., `--color-surface-container-high`, `--font-mono-data-family`). 

### Core Views
- **App Shell (`+layout.svelte`)**: Contains the `Sidebar`, the `<slot />` for content, and the `NowPlayingBar`. Also mounts `<NowPlayingExpanded />` as an overlay.
- **Albums (`/albums`)**: A responsive grid view of album cards with NO COVER fallbacks.
- **Tracks (`/tracks`)**: A detailed tabular list of tracks, formatting times with monospaced data fonts, and highlighting the currently playing row.
- **Artists (`/artists`)**: A grid of circular artist avatars, derived dynamically from the mock albums data to ensure consistency.
- **Playlists (`/playlists`)**: A grid of square playlist covers using a "stacked tracks" visual placeholder.
- **History (`/history`)**: Shows a list of recently played tracks. Uses an empty state (`Clock` icon, "No listening history yet") if none exist. Tracks are appended here when `play(track)` is called.
- **Subsonic (`/subsonic`)**: Displays an empty/disconnected state, instructing the user to configure a server connection.
- **Settings (`/settings`)**: A multi-panel view for configuring the server connection (URL, token, API version), viewing diagnostics (latency, sync status), and toggling advanced playback features (transcoding, Last.fm scrobbling) via custom flat toggle switches.
- **Expanded Now Playing (`NowPlayingExpanded.svelte`)**: A full-screen, two-column overlay triggered from the player bar. It features a custom native HTML5 Canvas color extractor that samples the album art to generate a dynamic, soft ambient background wash—keeping the app completely offline-capable without relying on external packages. It includes full transport controls, a progress bar, metadata, and an Up Next queue list.

## 6. Directory Structure
- `src-tauri/`: Rust backend and native integration.
  - `crates/app/`: App-specific Rust backend logic.
  - `src/`: Main Tauri entry point and configuration.
  - `tauri.conf.json`: Configuration for the native window (`backgroundColor: "#131318"`, custom titlebars via `decorations: false`).
- `src/`: SvelteKit Frontend source code.
  - `lib/components/`: Reusable UI components (`Sidebar.svelte`, `NowPlayingBar.svelte`, `ToggleSwitch.svelte`).
  - `lib/stores/`: Application state management (`player.ts`, `library.ts`, `server.ts`, `types.ts`).
  - `lib/theme/`: Design tokens (`tokens.css`) and local font declarations (`fonts.css`).
  - `routes/`: File-based client-side routing (`/albums`, `/tracks`, `/settings`).
- `static/fonts/`: Downloaded `.woff2` font files for offline use.

## 7. Change Log & Task History (Detailed)

### Task: Implement History Page and Click-to-Play
**Changes Made:**
1. **`src/lib/stores/player.ts`**:
   - Added `export const history = writable<Track[]>([]);` to store the playback history.
   - Added `export function play(track: Track): void` which updates `currentTrack`, sets `isPlaying: true`, resets `positionSeconds: 0`, and calls `logToHistory(track)`.
   - Added private `logToHistory(track: Track)` function which prepends the track to the `history` store, explicitly avoiding consecutive duplicate logs (`h[0].id === track.id`).
   - *Why*: To support history tracking as requested and replace the mock UI `togglePlay` with a functional `play(track)` action.
2. **`src/routes/tracks/+page.svelte`**:
   - Imported `play` from `$lib/stores/player`.
   - Modified the track table row to accept clicks: `<tr class:playing={isCurrent} on:click={() => play(track)}>` and added `cursor: pointer` to `.tracks-table tr`.
   - *Why*: So users can actually play a track from the library and populate the new history store.
3. **`src/routes/history/+page.svelte`**:
   - Created this new file to handle the `/history` route which was previously 404ing.
   - Imported the `history` store and `Clock` icon.
   - Implemented an empty state matching the `/subsonic` page pattern.
   - Implemented a table identical to the `/tracks` table to render the `$history` array. Replaced the track number with a `Clock` icon to denote history, and added the `on:click={() => play(track)}` behavior to allow re-playing from history.
   - *Why*: To complete the sidebar navigation and visually render the new history state.

### Task: Fix Layout Bugs in Expanded Now Playing View
**Changes Made:**
1. **`src/lib/components/NowPlayingExpanded.svelte`**:
   - Modified `.right-column` CSS: changed `flex: 1` to `flex: 1 1 auto; min-width: 0; max-width: 100%; overflow-x: hidden;`.
   - Modified `.track-title` CSS: added `overflow-wrap: break-word; word-break: break-word; white-space: normal;`.
   - *Why*: To fix an issue where long track titles forced the right column to overflow horizontally outside the viewport instead of wrapping text correctly, resolving clipping issues on narrower window sizes.
2. **`src/routes/+layout.svelte`**:
   - Imported `isExpandedViewOpen` from `$lib/stores/player`.
   - Wrapped the `<NowPlayingBar />` component in a conditional block: `{#if !$isExpandedViewOpen}`.
   - *Why*: To fix an issue where the persistent bottom player bar was overlapping with the bottom of the full-screen Expanded Now Playing overlay. Now, the small bar is removed from the DOM when the overlay is open, cleanly resolving the overlap.

### Task: Tauri Runtime Verification and Configuration Pass
**Changes Made:**
1. **Verification Attempt**:
   - Initiated the 7-step checklist to configure and run the existing UI in a native Tauri v2 desktop window via `npm run tauri dev`.
   - *Why*: To move testing from the Chrome browser tab to an actual native desktop window.
2. **Result: ABORTED at Step 1 (Previous Session)**:
   - Ran `rustc --version` and `cargo --version`. Both failed with "CommandNotFoundException" indicating the Rust toolchain is not installed or not in the PATH.
   - *Why*: The environment cannot run Tauri without Rust. Per instructions, this is a blocking issue and workarounds/auto-installation were strictly prohibited, halting the checklist process.
3. **Resumed Verification (Current Session)**:
   - User successfully installed Rust.
   - **Step 1:** Injected cargo bin path into process environment.
   - **Step 2-4:** Verified `svelte.config.js` (`adapter-static`), `+layout.ts` (`ssr = false`), and `tauri.conf.json` (`devUrl: http://localhost:1420`). All were correctly configured.
   - **Step 5 (Fixes):** Installed missing `@tauri-apps/cli` as a devDependency. Added missing `[package]` configuration to `src-tauri/Cargo.toml` and removed empty workspace members that were causing the Tauri builder to crash. Created the required Tauri bootstrapping files (`build.rs` and `src/main.rs`) since they were missing.
   - **Step 6:** Verified `WindowControls.svelte` imports and uses the real `@tauri-apps/api/window` methods, and confirmed `Sidebar.svelte` possesses the `data-tauri-drag-region` attribute.
   - *Why*: To resolve all build configuration and structural errors preventing Tauri from successfully launching the frontend.
4. **Environment Refresh & Continued Troubleshooting**:
   - After IDE restart, `cargo` still wasn't detected in the terminal's PATH despite being in the User PATH. This is a known Windows issue where IDE processes spawned before the PATH change don't inherit the updated value.
   - *Fix*: Injected `$env:USERPROFILE\.cargo\bin` into the process PATH at runtime for each command.
5. **File Lock Errors During Compilation (`os error 32`)**:
   - Cargo failed repeatedly with `The process cannot access the file because it is being used by another process` during parallel compilation. Multiple crates (`syn`, `icu_normalizer_data`, `web_atoms`) were affected.
   - Root cause: The IDE's file watcher was scanning and indexing files in `src-tauri/target/` as cargo wrote them, creating file lock contention.
   - *Fix 1*: Created `.gitignore` files (root and `src-tauri/`) to exclude `target/`, `node_modules/`, and build outputs from IDE file watching.
   - *Fix 2*: Set `CARGO_BUILD_JOBS=1` to compile crates sequentially, eliminating the race condition between parallel cargo threads and the file watcher.
   - *Why*: Windows file locking is more aggressive than Unix — any process reading a file can block another from writing/deleting it.
6. **Missing Application Icons**:
   - After all 367 dependencies compiled successfully, the final build script (`tauri-build`) failed with: `icons/icon.ico not found; required for generating a Windows Resource file`.
   - *Fix*: Generated all required icon files programmatically via a Node.js script (`gen_icons.cjs`): `32x32.png`, `128x128.png`, `128x128@2x.png`, `icon.ico`, and `icon.icns`. The icons use a monochrome concentric-ring/sound-wave design on the `#131318` dark background.
   - Script was deleted after use; icons persist in `src-tauri/icons/`.
   - *Why*: Tauri requires icon assets to embed into the Windows executable as a resource.
7. **✅ SUCCESSFUL LAUNCH**:
   - `npm run tauri dev` completed successfully. Vite dev server started on `localhost:1420`, Rust compiled in 37s (cached deps), and `project-resonate.exe` launched as a native desktop window.
   - Vite optimized `lucide-svelte` and `@tauri-apps/api/window` on first load.
   - Minor: `[404] GET /favicon.png` — favicon not yet created (non-blocking).

### Task: Fix Window Controls and Drag Region in Native Tauri Window
**Changes Made:**
1. **Audit of `WindowControls.svelte`**:
   - Confirmed the component already correctly imports `getCurrentWindow` from `@tauri-apps/api/window` and wires `minimize()`, `toggleMaximize()`, and `close()` to the three buttons with proper `on:click` handlers.
   - Confirmed `-webkit-app-region: no-drag` is set on the `.window-controls` container to prevent drag region overlap.
   - *Result*: No changes needed — code was already correct.
2. **Audit of `Sidebar.svelte`**:
   - Confirmed `data-tauri-drag-region` attribute is present on both the `<aside class="sidebar">` and `<div class="brand">` elements.
   - Confirmed `-webkit-app-region: drag` is set on `.brand` CSS, and `-webkit-app-region: no-drag` is set on `.nav-item` to prevent nav buttons from being swallowed by the drag region.
   - *Result*: No changes needed — code was already correct.
3. **Audit of `@tauri-apps/api` version**:
   - Confirmed `package.json` has `"@tauri-apps/api": "^2.0.0"` — correct Tauri v2 API with `getCurrentWindow` support.
   - *Result*: No changes needed.
4. **Audit of `tauri.conf.json`**:
   - Confirmed `decorations: false` (custom titlebar), no conflicting `dragDropEnabled` settings.
   - *Result*: No changes needed.
5. **[NEW] `src-tauri/capabilities/default.json`** ← **ROOT CAUSE FIX**:
   - **This file did not exist.** Tauri v2 requires explicit capability/permission grants for ALL window API calls. Without this file, `minimize()`, `toggleMaximize()`, `close()`, and `startDragging()` all resolve silently without error but do absolutely nothing.
   - Created the file with permissions: `core:default`, `core:window:default`, `core:window:allow-minimize`, `core:window:allow-toggle-maximize`, `core:window:allow-close`, `core:window:allow-start-dragging`, plus comprehensive window management and shell plugin permissions.
   - *Why*: Tauri v2 moved from a blanket allowlist to a granular capability system. Every IPC call from the frontend to the Rust backend must be explicitly permitted, or it silently fails. This was the sole reason window controls and drag were non-functional.
6. **Process cleanup**:
   - Killed stale `project-resonate.exe` processes and freed port 1420 before rebuilding.
   - *Why*: Windows locks running executables — cargo cannot overwrite an exe that's still in memory.

### Task: Reposition Window Controls to Windows-Standard Top-Right Titlebar
**Changes Made:**
1. **`src/routes/+layout.svelte`**:
   - Added a new `<div class="titlebar" data-tauri-drag-region>` as the first child of `.app-shell`, sitting above the `.app-body` (sidebar + content).
   - The titlebar spans the full window width at `var(--titlebar-height, 32px)` height, uses `justify-content: flex-end` to push `<WindowControls />` to the far right, and has `-webkit-app-region: drag` so the entire strip is draggable.
   - `WindowControls` import moved here from Sidebar.
   - *Why*: Previously the controls were embedded in the sidebar's brand row (macOS-style left placement). Windows convention requires minimize/maximize/close at the absolute top-right corner of the window, spanning the full width — not nested inside a sidebar column.
2. **`src/lib/components/Sidebar.svelte`**:
   - Removed the `import WindowControls` and `<WindowControls />` usage from the brand row.
   - Removed the `.brand-row` wrapper div and CSS (no longer needed since it only existed to flex-space the wordmark and controls).
   - The brand section now simply contains the "RESONATE" wordmark and version label.
   - *Why*: The controls no longer belong in the sidebar — they're now in the layout-level titlebar.
3. **`src/lib/components/WindowControls.svelte`**:
   - Restyled buttons to match Windows 10/11 native titlebar conventions:
     - Buttons are now `46px` wide × `height: 100%` (fills titlebar height), with `border-radius: 0` (flat rectangles, not rounded).
     - Zero gap between buttons (flush edge-to-edge).
     - Close button hover: vivid Windows red (`#e81123`) with white icon — matching the system close button exactly.
     - Minimize/maximize hover: subtle `--color-surface-container-high` background.
   - Added `-webkit-app-region: no-drag` directly on each `.control-btn` as well as the container, ensuring buttons remain clickable within the drag-region titlebar.
   - Button order confirmed correct: minimize → maximize → close (left-to-right).
   - *Why*: The previous styling (28px rounded squares with `--color-error-container` close hover) was macOS-flavored. Windows users expect flat rectangular buttons with the standard red close hover.
4. **`src/lib/theme/tokens.css`**:
   - Added `--titlebar-height: 32px` to the Layout section.
   - *Why*: Centralizes the titlebar height as a design token so it can be referenced consistently by the layout and any future components that need to account for the titlebar offset.

### Task: Push Project to GitHub
**Changes Made:**
1. Initialized git repository in `c:\the vision` with `git init`.
2. Staged all 80 project files with `git add -A` — `.gitignore` correctly excluded `node_modules/`, `src-tauri/target/`, `.svelte-kit/`, and `build/`.
3. Configured local git identity (`Anu8hav` / noreply email) since no global git config existed.
4. Created initial commit with full project state: SvelteKit frontend, Tauri v2 backend, design system, all routes, mock stores, custom window controls.
5. Renamed default branch from `master` to `main`.
6. Added remote: `https://github.com/Anu8hav/Resonate.git`
7. Force-pushed to `origin/main` (remote had a pre-existing commit from repo creation — likely a default README).
- *Why*: User requested pushing the existing project to a newly created GitHub repository before beginning the Rust backend implementation work.
- **Repository URL**: https://github.com/Anu8hav/Resonate

## 8. Current State & Pending Tasks
- [x] Establish initial project architecture and feature roadmap.
- [x] Complete UI design system rebuild and layout implementation.
- [x] Mock data models and stores for Tracks, Albums, Player, and Settings.
- [x] Implement routing for all sidebar items (Albums, Tracks, Artists, Playlists, Subsonic, Settings).
- [x] Build Expanded Now Playing overlay with native Canvas-based dynamic ambient color extraction.
- [x] Implement History page with empty states and `play(track)` integration in Player store.
- [x] **Launch Tauri v2 desktop window with full SvelteKit frontend rendering.**
- [x] **Fix window controls (minimize/maximize/close) and drag region via Tauri v2 capabilities.**
- [x] **Reposition window controls to Windows-standard top-right titlebar strip.**
- [x] **Push project to GitHub** (https://github.com/Anu8hav/Resonate)
- [x] **Implement Rust backend for local file scanning and indexing.**
- [x] **Connect SvelteKit frontend to Rust backend via Tauri IPC (`invoke`).**
- [ ] Implement actual audio playback engine (CPAL/Symphonia via Rust).
- [ ] Integrate Subsonic API connection for remote streaming.

## 9. Local Library Backend Implementation
- **Dependencies**: Added `lofty` for metadata extraction, `rusqlite` (bundled) for SQLite, `walkdir` for directory scanning, `uuid` (v4), and `tauri-plugin-dialog` for native folder picking.
- **Database**: Created `db.rs` to initialize `resonate.db` in `app_data_dir()`. Implemented schema with `artists`, `albums`, `tracks`, and `scan_folders` tables. Included deduplication logic based on `file_path` for tracks and case-insensitive matching for artists and albums.
- **Scanner**: Created `scanner.rs` to recursively scan directories for `.mp3`, `.flac`, `.wav`, `.m4a`, and `.ogg` files. Uses `lofty` to extract metadata and falls back gracefully for missing tags.
- **IPC Commands**: Registered `pick_music_folder`, `scan_library`, `get_all_albums`, and `get_all_tracks` commands in `main.rs`. Fixed a compilation error regarding `to_string_lossy` on `tauri_plugin_dialog::FilePath` by using `.into_path().unwrap().to_string_lossy().to_string()`.
- **Frontend Integration**: Wired `library.ts` to call IPC commands, updated `+layout.svelte` to fetch library on startup, and added an "Add Music Folder" button with a scan summary inside the Settings > Library tab. Made `album` nullable on the `Track` interface and updated `tracks/+page.svelte` to handle null albums gracefully.
