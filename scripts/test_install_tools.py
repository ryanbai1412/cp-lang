"""scripts/install-tools.sh: every tool CI pins has a release on every runner, and a
miss installs that release's binary into DIR/bin."""
import glob
import io
import os
import re
import shutil
import subprocess
import tarfile
import tempfile
import unittest
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
SCRIPT = ROOT / "scripts" / "install-tools.sh"
RUNNERS = [("Linux", "X64"), ("macOS", "ARM64"), ("Windows", "X64")]


def pinned_tools():
    tools = set()
    for path in glob.glob(str(ROOT / ".github" / "**" / "*.yml"), recursive=True):
        text = Path(path).read_text(encoding="utf-8")
        for m in re.finditer(r"uses: \./\.github/actions/install-tools\n\s+with:\n\s+tools: (.+)\n", text):
            tools |= set(m.group(1).split())
    return tools


def release(tool, os_name, arch):
    return subprocess.run(["bash", str(SCRIPT), "--url", tool, os_name, arch],
                          capture_output=True, text=True)


class InstallToolsTest(unittest.TestCase):
    def test_every_pinned_tool_has_a_release_on_every_runner(self):
        tools = pinned_tools()
        self.assertIn("just@1.59.0", tools)
        for tool in sorted(tools):
            name, version = tool.split("@")
            for os_name, arch in RUNNERS:
                r = release(tool, os_name, arch)
                self.assertEqual(r.returncode, 0, f"{tool} on {os_name}: {r.stderr}")
                url, binary = r.stdout.split()
                self.assertTrue(url.startswith("https://github.com/"), url)
                self.assertIn(version, url)
                self.assertEqual(binary, name + (".exe" if os_name == "Windows" else ""))

    def test_a_tool_without_a_recipe_fails_naming_where_to_add_it(self):
        r = release("cargo-binstall@1.0.0", "Linux", "X64")
        self.assertNotEqual(r.returncode, 0)
        self.assertIn("add one to scripts/install-tools.sh", r.stderr)

    def test_a_miss_puts_the_release_binary_in_dir_bin(self):
        tmp = Path(tempfile.mkdtemp())
        self.addCleanup(shutil.rmtree, tmp)
        archive = tmp / "just.tar.gz"
        with tarfile.open(archive, "w:gz") as tar:
            body = b"#!/bin/sh\necho just 1.59.0\n"
            info = tarfile.TarInfo("just-1.59.0/just")
            info.size, info.mode = len(body), 0o644
            tar.addfile(info, io.BytesIO(body))
        fake = tmp / "fake"
        fake.mkdir()
        # A curl that serves the archive for whatever URL it is given, at its -o path.
        (fake / "curl").write_text(
            '#!/bin/sh\nwhile [ $# -gt 0 ]; do [ "$1" = -o ] && { out=$2; shift; }; shift; done\n'
            f'cp "{archive}" "$out"\n')
        (fake / "curl").chmod(0o755)
        env = {**os.environ, "PATH": f"{fake}{os.pathsep}{os.environ['PATH']}",
               "RUNNER_OS": "Linux", "RUNNER_ARCH": "X64"}
        r = subprocess.run(["bash", str(SCRIPT), str(tmp / "tools"), "just@1.59.0"],
                           env=env, capture_output=True, text=True)
        self.assertEqual(r.returncode, 0, r.stderr)
        installed = tmp / "tools" / "bin" / "just"
        self.assertTrue(os.access(installed, os.X_OK))
        self.assertEqual(subprocess.run([str(installed)], capture_output=True, text=True).stdout,
                         "just 1.59.0\n")

    def test_the_action_saves_a_miss_before_the_job_can_fail(self):
        action = (ROOT / ".github/actions/install-tools/action.yml").read_text(encoding="utf-8")
        self.assertIn("actions/cache/restore@v4", action)
        self.assertIn("actions/cache/save@v4", action)
        self.assertNotRegex(action, r"uses: actions/cache@")
        self.assertIn("scripts/install-tools.sh", action)
        # A changed recipe misses, so the next run proves it on every runner.
        self.assertIn("hashFiles('scripts/install-tools.sh')", action)


if __name__ == "__main__":
    unittest.main()
