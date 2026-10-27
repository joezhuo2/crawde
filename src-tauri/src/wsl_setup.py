"""Invoked inside each WSL distro; stdin contains setup options, never shell code."""
import json
import os
from pathlib import Path
import shlex
import shutil
import subprocess
import sys
import time


def merge(settings, events, command, install):
    hooks = settings.setdefault("hooks", {})
    for event in events:
        groups = hooks.setdefault(event, [])
        for group in groups:
            group["hooks"] = [h for h in group.get("hooks", []) if not (
                h.get("url") == "http://127.0.0.1:47631/events"
                or "--crawde-hook" in h.get("args", [])
                or "--crawde-hook" in h.get("command", "")
            )]
        groups[:] = [g for g in groups if g.get("hooks")]
        if install:
            groups.append({"hooks": [{"type": "command", "command": command, "timeout": 3}]})
    return settings


def main():
    options = json.load(sys.stdin)
    directory = Path(os.environ.get("CLAUDE_CONFIG_DIR") or Path.home() / ".claude")
    # Do not create Claude configuration in distros that have never used Claude.
    if not directory.is_dir():
        return
    path = directory / "settings.json"
    original = path.read_text() if path.exists() else "{}"
    settings = json.loads(original)
    executable = subprocess.check_output(["wslpath", "-u", options["executable"]], text=True).strip()
    command = shlex.join([executable, "--crawde-hook", "--wsl-distro", os.environ["WSL_DISTRO_NAME"]])
    merge(settings, options["events"], command, options["install"])
    updated = json.dumps(settings, indent=2) + "\n"
    if updated == original:
        return
    if path.exists():
        shutil.copy2(path, directory / f"settings.crawde-{time.time_ns()}.bak")
    staging = directory / "settings.crawde.tmp"
    staging.write_text(updated)
    staging.replace(path)


if __name__ == "__main__":
    main()
