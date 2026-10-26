# Changelog

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
