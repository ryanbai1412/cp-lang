"""One command per task (ci/R36): every `just <recipe>` a live doc names
exists, every recipe says what it does in `just --list`, and every command in
infra's "Suites and tools" table (the contributor
workflow) has a recipe that runs it."""
import glob
import os
import re
import unittest

ROOT = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
HISTORY = ("research/", "corpus/", "plan/archive/")


def recipes(justfile):
    """{name: (has_doc_comment, body)} for each recipe in a justfile."""
    out, lines = {}, justfile.split("\n")
    for i, line in enumerate(lines):
        if m := re.match(r"([a-z][\w-]*)(?:\s[^:=]*)?:(?!=)", line):
            body = []
            for b in lines[i + 1:]:
                if not b.startswith((" ", "\t")):
                    break
                body.append(b.strip())
            out[m.group(1)] = (i > 0 and lines[i - 1].startswith("#"), "\n".join(body))
    return out


def unknown_recipes(docs, names):
    return sorted({f"{path}: just {r}" for path, text in docs
                   for r in re.findall(r"`just ([a-z][\w-]*)", text) if r not in names})


def table_commands(text):
    out = []
    for part in re.split(r"(?m)^## ", text)[1:]:
        if part.startswith("Suites and tools"):
            out += re.findall(r"(?m)^\| `(?:python3 |uv run python )?(?:-m unittest )?(scripts/[\w./*-]+?)[` ]", part)
    return out


def tasks_without_recipe(commands, rs):
    bodies = "\n".join(b for _, b in rs.values())
    return [c for c in commands if c.replace("test_*", "test_") not in bodies.replace("test_*", "test_")]


class JustfileTest(unittest.TestCase):
    def setUp(self):
        with open(os.path.join(ROOT, "justfile"), encoding="utf-8") as f:
            self.recipes = recipes(f.read())

    def test_docs_name_only_existing_recipes(self):
        docs = []
        for p in glob.glob(os.path.join(ROOT, "**", "*.md"), recursive=True):
            rel = os.path.relpath(p, ROOT)
            if not rel.startswith(HISTORY + ("node_modules", "target", "web/node_modules")) and "/node_modules/" not in rel:
                with open(p, encoding="utf-8", errors="replace") as f:
                    docs.append((rel, f.read()))
        self.assertEqual(unknown_recipes(docs, self.recipes), [], "add the recipe to justfile or fix the doc (ci/R36)")

    def test_every_recipe_says_what_it_does(self):
        self.assertEqual([n for n, (doc, _) in self.recipes.items() if not doc and n != "default"], [],
                         "put a `# ...` comment above each recipe so `just --list` explains it (ci/R36)")

    def test_every_handbook_task_has_a_recipe(self):
        with open(os.path.join(ROOT, "handbook", "domains", "infra.md"), encoding="utf-8") as f:
            commands = table_commands(f.read())
        self.assertGreater(len(commands), 5)
        self.assertEqual(tasks_without_recipe(commands, self.recipes), [],
                         "handbook/domains/infra.md: add a justfile recipe that runs it (ci/R36)")

    def test_flags_a_missing_recipe_and_an_unlisted_task(self):
        rs = recipes("# Land.\nland:\n    scripts/land.sh\n\nbare:\n    true\n")
        self.assertEqual(set(rs), {"land", "bare"})
        self.assertFalse(rs["bare"][0])
        self.assertEqual(unknown_recipes([("README.md", "run `just land` then `just ship`")], rs),
                         ["README.md: just ship"])
        table = "## Suites and tools\n\n| `scripts/land.sh` | x |\n| `python3 scripts/culprit.py` | y |\n"
        self.assertEqual(tasks_without_recipe(table_commands(table), rs), ["scripts/culprit.py"])


if __name__ == "__main__":
    unittest.main()
