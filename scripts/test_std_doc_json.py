"""std-docs/R5: `cpl doc --format json std` shows every public fn's trait bounds and defaults,
so hover, completion and the site (all fed by it) show them too."""
import json
import os
import re
import subprocess
import unittest
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
CPL = os.environ.get("CPL", str(ROOT / "target" / "debug" / "cpl"))
BOUND = re.compile(r"\b\w+\s*:\s*[A-Z]\w*(?:<[^<>]*>)?(?:\s*\+\s*[A-Z]\w*)*(?=[,>])")
DEFAULT = re.compile(r"\b(\w+)\s*:\s*[^,()=]+?=\s*[^,)]+")


def head(line):
    """The source's signature through its parameter list."""
    line = line.strip()
    start = line.find("(")
    depth = 0
    for i in range(start, len(line)):
        depth += {"(": 1, ")": -1}.get(line[i], 0)
        if depth == 0:
            return line[:i + 1]
    return line


def public_fns(doc):
    for m in doc["modules"]:
        source = (ROOT / m["path"]).read_text().split("\n")
        for item in m["items"]:
            for it in [item, *item.get("members", [])]:
                if it["kind"] == "fn" and "pub " in it["signature"] and it.get("line"):
                    yield m["path"], it, source[it["line"] - 1]


class DocJsonShape(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        out = subprocess.run([CPL, "doc", "--format", "json", "std"], cwd=ROOT,
                             capture_output=True, text=True, check=True).stdout
        cls.fns = list(public_fns(json.loads(out)))

    def test_every_public_fn_is_listed(self):
        self.assertGreater(len(self.fns), 1000)

    def test_signatures_keep_bounds_and_defaults(self):
        missing = []
        for path, it, line in self.fns:
            src = head(line)
            sig = re.sub(r"\s+", " ", it["signature"])
            for want in BOUND.findall(src) + [d.group(0).split("=")[0].strip() for d in DEFAULT.finditer(src)]:
                if re.sub(r"\s+", " ", want) not in sig:
                    missing.append(f"{path}:{it['line']}: `{want}` not in `{sig}`")
        self.assertEqual(missing, [])


if __name__ == "__main__":
    unittest.main()
