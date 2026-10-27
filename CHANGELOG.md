# Changelog

## 0.1.15 — 2026-10-27

- Briefly wake every 1–4 minutes while sleeping, walking and performing 1–3 attacks over 15–45 seconds before sleeping again.
- Respect Little walks and reduced motion, and cancel the cycle when Claude activity resumes.

## 0.1.14 — 2026-10-27

- Animate only the crab during little walks, keeping the native window and session cards anchored.
- Wander left and right within 100 pixels of the placed position, with 4–8 second pauses while Claude is working.
- Keep walks within the window and pause on hover, drag, keyboard focus, sleep, or reduced motion; pinned cards can stay open while the crab walks.

## 0.1.13 — 2026-10-27

- Remove the new-chat and notification action buttons beneath the crab.
- Remove the empty control space and its top-edge hover offset; chats still expand on hover and can be pinned.

## 0.1.12 — 2026-10-27

- Add an accessible × button to dismiss notification and error banners.
- Keep dismissed messages hidden while allowing new errors or retried actions to show feedback again.

## 0.1.11 — 2026-10-27

- Move top-docked chat cards up into the hidden action-button space while collapsed.
- Slide cards down smoothly when controls appear, respecting reduced motion.

## 0.1.10 — 2026-10-27

- Clicking the notification bell restores chats dismissed with × and expands the chat list.
- Keep expired completed sessions out of the restored list.

## 0.1.9 — 2026-10-27

- Click the crab, or press Enter/Space while it is focused, to open Claude Desktop and play the crab ability.
- Keep dragging and cancelled gestures from opening Claude.

## 0.1.8 — 2026-10-27

- Install activity hooks in existing Claude configurations inside WSL distributions.
- Forward WSL events through Windows interop without exposing the receiver to the network.
- Resume WSL chats and create new chats in their original Linux distro and project.
- Wake the startup-hidden pet when WSL sessions become active.

## 0.1.7 — 2026-10-26

- Replace the two update menu items with one versioned Up to date / Install latest update action.
- Always check on launch, including when an older preference disabled update checks.
- Show checking, download, and retry states without reporting a failed check as up to date.

## 0.1.6 — 2026-10-26

- Move Reduce motion, Launch on startup, Little walks, and Sound effects exclusively to both right-click menus.
- Move hook installation, reinstallation, and disconnection into the menus.
- Keep menu checkmarks and open Settings windows synchronized with saved preferences.

## 0.1.5 — 2026-10-26

- Move Quit Crawde from Settings to both right-click menus.
- Save preference changes automatically, preserving other changes made through native controls.

## 0.1.4 — 2026-10-26

- Move Check for updates from Settings to both right-click menus.
- Add Close + install latest update with checksum verification and visible error status.

## 0.1.3 — 2026-10-26

- Fix the invisible crab in Windows installers by shipping sprite sheets as local PNG assets instead of inline data URLs blocked by the native Content Security Policy.
- Validate every production bundle against the native image policy and verify that all five sprite sheets are packaged and referenced.
- Apply the native Content Security Policy to production previews, so preview checks catch installer-only image failures.

## 0.1.2 — 2026-10-26

- Wake the pet from ongoing Claude activity, including long-running tools and embedded sessions.
- Replace the unsupported HTTP startup hook with a quiet command forwarder; repair older configurations automatically.
- Show the most recent received activity in Claude settings.
- Bring single-chat bubbles closer to the crab by removing unused pagination space and transparent sprite headroom.
- Refine sleep timing and drifting “zzz”; preserve the locally selected 5 fps idle speed.
- Add a dedicated high-resolution claw tray icon with 16/24/32/128px artwork, a matching banner, and reproducible branding sources.
- Restyle the README with quick navigation, live badges, setup and usage sections, and licensing information.
- Add tagged Windows releases with installer downloads, generated notes, and SHA-256 checksums.

## 0.1.0 — 2026-10-26

- First Windows Tauri 2 release with an animated crab, chat activity bubbles, Claude integration, tray controls, settings, sounds, and saved desktop positions.
