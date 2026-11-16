---
description: Install, remove, or check Crawde activity hooks
argument-hint: "[install|uninstall|status]"
allowed-tools: Bash, PowerShell
---

Manage the Crawde desktop companion's Claude Code hooks. The requested action is: `$ARGUMENTS` (default to `install` when empty).

1. Find the Crawde executable. Use the first that exists:
   - Windows: `$env:LOCALAPPDATA\Crawde\crawde.exe`, then `$env:ProgramFiles\Crawde\crawde.exe`.
   - WSL: the same Windows paths through `/mnt/c` (get `%LOCALAPPDATA%` with `cmd.exe /c echo %LOCALAPPDATA%` and convert it with `wslpath`).
   - macOS: `/Applications/Crawde.app/Contents/MacOS/crawde`.
   - Otherwise, read the `SessionStart` hook in `~/.claude/settings.json` whose args include `--crawde-hook`; its `command` is the executable.
   If none exists, say Crawde is not installed and link https://github.com/joezhuo2/crawde/releases/latest. Stop there.
2. Run it with exactly one flag: `install` → `--install-hooks`, `uninstall` → `--uninstall-hooks`, `status` → `--hook-status`. Any other argument: explain the three choices and stop.
3. Report the printed message in one line. After an install or uninstall, remind the user to restart Claude Code sessions so the change takes effect.

Do not edit `settings.json` yourself; the executable backs it up and preserves other hooks.
