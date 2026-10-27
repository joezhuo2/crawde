"""Run with Python 3 on Linux/WSL or Windows; never edits real Claude settings."""
import importlib.util
from pathlib import Path
import unittest
import json
import os
import subprocess
import tempfile

spec = importlib.util.spec_from_file_location("setup", Path(__file__).resolve().parents[1] / "src-tauri/src/wsl_setup.py")
setup = importlib.util.module_from_spec(spec)
spec.loader.exec_module(setup)


class Hooks(unittest.TestCase):
    @unittest.skipUnless(os.environ.get("WSL_DISTRO_NAME"), "Requires WSL")
    def test_real_wsl_setup_backup_idempotence_and_malformed_settings(self):
        with tempfile.TemporaryDirectory(prefix="crawde hooks ") as temporary:
            path = Path(temporary) / "settings.json"
            original = '{"theme":"dark"}'
            path.write_text(original)
            environment = dict(os.environ, CLAUDE_CONFIG_DIR=temporary)
            options = {"executable": r"C:\Program Files\Crawde\crawde.exe", "events": ["SessionStart", "Stop"], "install": True}
            def run():
                return subprocess.run(["python3", spec.origin], input=json.dumps(options), text=True, env=environment, capture_output=True)
            self.assertEqual(run().returncode, 0)
            self.assertEqual(next(Path(temporary).glob("*.bak")).read_text(), original)
            self.assertIn("/mnt/c/Program Files/Crawde/crawde.exe", json.loads(path.read_text())["hooks"]["Stop"][0]["hooks"][0]["command"])
            self.assertEqual(run().returncode, 0)
            self.assertEqual(len(list(Path(temporary).glob("*.bak"))), 1)
            options["install"] = False
            self.assertEqual(run().returncode, 0)
            self.assertEqual(json.loads(path.read_text())["hooks"]["Stop"], [])
            path.write_text("malformed")
            self.assertNotEqual(run().returncode, 0)
            self.assertEqual(path.read_text(), "malformed")

    def test_install_repair_and_disconnect_preserve_other_handlers(self):
        settings = {"theme": "dark", "hooks": {"SessionStart": [{"matcher": "*", "hooks": [
            {"type": "command", "command": "keep-me"},
            {"type": "http", "url": "http://127.0.0.1:47631/events"},
        ]}]}}
        events = ["SessionStart", "Stop"]
        command = "'/mnt/c/Program Files/Crawde/crawde.exe' --crawde-hook --wsl-distro Ubuntu-24.04"
        setup.merge(settings, events, command, True)
        setup.merge(settings, events, command, True)
        self.assertEqual(len(settings["hooks"]["SessionStart"]), 2)
        self.assertEqual(settings["hooks"]["Stop"][0]["hooks"][0]["command"], command)
        setup.merge(settings, events, command, False)
        self.assertEqual(settings["hooks"]["SessionStart"], [{"matcher": "*", "hooks": [{"type": "command", "command": "keep-me"}]}])
        self.assertEqual(settings["hooks"]["Stop"], [])
        self.assertEqual(settings["theme"], "dark")


if __name__ == "__main__":
    unittest.main()
