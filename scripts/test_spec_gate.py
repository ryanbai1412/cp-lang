#!/usr/bin/env python3
"""One test per gate behind scripts/spec-gate.py."""
import importlib.util
import os
import pathlib
import sys
import tempfile
import json
import unittest
from pathlib import Path
from unittest import mock

HERE = pathlib.Path(__file__).resolve().parent
sys.path.insert(0, str(HERE))
SPEC = importlib.util.spec_from_file_location("spec_gate", HERE / "spec-gate.py")
assert SPEC and SPEC.loader
GATE = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(GATE)

import corpus  # noqa: E402
import specgate  # noqa: E402


class Tree:
    """A scratch `spec/conformance/` the gates read instead of the repo's."""

    def __init__(self, files: dict[str, str]):
        self.tmp = tempfile.TemporaryDirectory()
        self.root = pathlib.Path(self.tmp.name)
        self.conf = self.root / "spec" / "conformance"
        for rel, text in files.items():
            p = self.root / rel
            p.parent.mkdir(parents=True, exist_ok=True)
            p.write_bytes(text.encode())
        self.patches = [mock.patch.object(specgate, "ROOT", self.root),
                        mock.patch.object(specgate, "CONFORMANCE", self.conf)]

    def __enter__(self):
        for p in self.patches:
            p.start()
        return self

    def __exit__(self, *exc):
        for p in self.patches:
            p.stop()
        self.tmp.cleanup()

    def read(self, rel: str) -> str:
        return (self.root / rel).read_bytes().decode()


def run(name: str, *argv: str):
    import argparse
    gate = GATE.load(name)
    p = argparse.ArgumentParser()
    gate.args(p)
    return gate.run(p.parse_args(list(argv)))


class GateListTest(unittest.TestCase):
    def test_every_gate_has_a_test_here(self):
        tested = {n[len("test_"):].replace("_", "-") for n in dir(GatesTest) if n.startswith("test_")}
        tested |= {t.split("-rejects")[0] for t in tested}
        self.assertEqual(set(GATE.GATES) - tested, set())


class GatesTest(unittest.TestCase):
    def test_changelog(self):
        files = {"spec/diagnostics.md": "| E0503 | removed | `a.size` is `a.len`, `pop_front` is `pop` |\n",
                 "CHANGELOG.md": "# Changelog\n\n## 0.3\n\n- `a.size` is gone\n\n## 0.2\n\n- `pop_front`\n"}
        with Tree(files):
            self.assertEqual([p.what.split(" ")[0] for p in run("changelog")], ["`pop_front`"])

    def test_decisions_explained(self):
        files = {"spec/core/a.md": "Per decision 12 and decision 345.\n",
                 "handbook/language/b.md": "Decision 345 chose this.\n"}
        with Tree(files):
            self.assertEqual([p.where for p in run("decisions-explained")], ["decision 12"])

    def test_vocab(self):
        files = {"std/a.cp": "pub fn f() -> ?Int = some(1)\n/// some(x) in a doc is prose\n",
                 "spec/core/b.md": "Mint in prose\n```\nlet m: Mint = 1\n```\n"}
        with Tree(files):
            self.assertEqual(sorted(p.where for p in run("vocab")), ["spec/core/b.md:1", "spec/core/b.md:3", "std/a.cp:1"])

    def test_xref(self):
        import subprocess
        files = {"spec/core/01-a.md": "A rule (R01.1.1). A permit (R01.1.3, permits).\n",
                 "handbook/x.md": "See R01.1.1, R01.1.3 and R01.1.2.\n"}
        with Tree(files) as t:
            subprocess.run(["git", "init", "-q"], cwd=t.root, check=True)
            subprocess.run(["git", "add", "."], cwd=t.root, check=True)
            self.assertEqual([(p.where, p.what[:7]) for p in run("xref")], [("handbook/x.md:1", "R01.1.2")])

    def test_flakes(self):
        from specgate import flakes
        verdicts = iter([{"a.cp", "b.cp"}, {"a.cp"}])
        with mock.patch.object(flakes, "failing", lambda f: next(verdicts)):
            self.assertEqual([(p.where, p.what) for p in run("flakes")], [("b.cp", "failed on 1 of 2 runs")])

    def test_speed(self):
        with Tree({}):
            self.assertEqual(run("speed", "--cpl-conform", "true", "--budget", "1e9"), [])
            self.assertEqual(len(run("speed", "--cpl-conform", "true", "--budget", "-1")), 1)
            self.assertIn("exited 2", run("speed", "--cpl-conform", "sh", "--budget", "1e9")[0].what)

    def test_tool_rules(self):
        files = {"spec/tools/a.md": "- `cpl x` does x (Rx.1).\n- `cpl y` does\n  y.\n\n```\n- not a feature\n```\n"}
        with Tree(files):
            self.assertEqual([p.where for p in run("tool-rules")], ["spec/tools/a.md:2"])

    def test_widths(self):
        from specgate import widths
        self.assertEqual(widths.exact("div", -7, 2), -4)
        self.assertEqual(widths.wrap(2**63), -2**63)
        with Tree({"spec/conformance/widths/old.cp": "x"}) as t:
            self.assertEqual(len(run("widths")), len(widths.cases()) + 1)
            self.assertEqual(run("widths", "--write"), [])
            self.assertFalse((t.conf / "widths/old.cp").exists())
            self.assertIn("print(id(1) + id(2))", t.read("spec/conformance/widths/add-in-range.cp"))
            self.assertEqual(run("widths"), [])

    def test_matrix(self):
        toml = ('[m.col.c]\ncode = "print($v)"\nout = "$v"\n[m.col.d]\ncode = "print($v + 1)"\nout = "$w"\n'
                '[m.row.r]\nv = 2\nw = 3\n[m.row.s]\nv = 5\n[m.na]\n"s--d" = "no w"\n')
        with Tree({"spec/conformance/matrix.toml": toml, "spec/conformance/matrix/m/old.cp": "x"}) as t:
            self.assertEqual(sorted(p.where.split("/")[-1] for p in run("matrix")), ["old.cp", "r--c.cp", "r--d.cp", "s--c.cp"])
            self.assertEqual(run("matrix", "--write"), [])
            self.assertEqual(t.read("spec/conformance/matrix/m/r--d.cp"), '// expect: ok, output "3"\nprint(2 + 1)\n')
            self.assertFalse((t.conf / "matrix/m/old.cp").exists())
            self.assertEqual(run("matrix"), [])

    def test_ownership(self):
        from specgate import ownership
        cases = ownership.cases()
        self.assertTrue(cases["top-temp-let.cp"].startswith("// expect: E0603\n"))
        self.assertTrue(cases["fn-named-let.cp"].startswith('// expect: ok, output "4"\n'))
        with Tree({}) as t:
            self.assertEqual(len(run("ownership")), len(cases))
            self.assertEqual(run("ownership", "--write"), [])
            self.assertEqual(run("ownership"), [])

    def test_traits(self):
        files = {"std/a.cp": "pub trait Shout:\n    fn shout(self) -> Str\n\npub struct Dog(n: Int)\nimpl Shout for Dog:\n    fn shout(self) -> Str = \"woof\"\n",
                 "spec/conformance/traits.toml": '[case."Shout/Dog/shout"]\ncode = "print(Dog(1).shout())"\nout = "woof"\n'}
        files["crates/cpl-check/frozen/std/a.cp"] = files["std/a.cp"]
        with Tree(files) as t:
            self.assertEqual([p.where.split("/")[-1] for p in run("traits")], ["Dog--shout.cp"])
            self.assertEqual(run("traits", "--write"), [])
            self.assertEqual(t.read("spec/conformance/traits/Shout/Dog--shout.cp"), '// expect: ok, output "woof"\nprint(Dog(1).shout())\n')
        with Tree({"std/a.cp": files["std/a.cp"], "spec/conformance/traits.toml": ""}):
            self.assertEqual([p.what for p in run("traits")], ["no case: Shout/Dog/shout (write one)"])
        unpinned = {"std/a.cp": files["std/a.cp"], "spec/conformance/traits.toml": files["spec/conformance/traits.toml"]}
        with Tree(unpinned) as t:
            self.assertEqual(run("traits", "--write"), [])
            self.assertIn("// engines: next\n", t.read("spec/conformance/traits/Shout/Dog--shout.cp"))
        next_only = files["spec/conformance/traits.toml"] + 'engines = "next"\n'
        with Tree({"std/a.cp": files["std/a.cp"], "spec/conformance/traits.toml": next_only}) as t:
            self.assertEqual(run("traits", "--write"), [])
            self.assertEqual(t.read("spec/conformance/traits/Shout/Dog--shout.cp"),
                             '// expect: ok, output "woof"\n// engines: next\nprint(Dog(1).shout())\n')

    def test_prelude(self):
        files = {"std/prelude.cp": "use std.a.{f}\nuse std.b.{k}\n",
                 "std/a.cp": "pub fn f() -> Int = 1\npub fn g() -> Int = 2\npub fn h() -> Int = 3\npub fn m(self) -> Int = 4\n",
                 "std/b.cp": "pub fn inv() -> Int = 1\npub fn k() -> Int = 2\n",
                 "spec/lib/a.md": "```cp\nfn f() -> Int\nfn g() -> Int\nfn m(self) -> Int\nfn b.inv() -> Int\n```\n| `h(x)` | h |\n"}
        with Tree(files) as t:
            self.assertEqual([p.what.split(" ")[0] for p in run("prelude")], ["`g`", "`h`"])
            self.assertFalse((t.root / "spec/conformance/prelude-gaps.txt").exists())
            (t.root / "std/prelude.cp").write_text("use std.a.{f, g, h}\nuse std.b.{inv, k}\n")
            self.assertEqual([p.what.split(" ")[0] for p in run("prelude")], ["`b.inv`"])
            (t.root / "std/prelude.cp").write_text("use std.a.{f, g, h}\nuse std.b.{k}\n")
            self.assertEqual(run("prelude"), [])
            (t.root / "spec/lib/a.md").write_text("```cp\nfn b.nope() -> Int\n```\n")
            self.assertEqual([p.what.split(" ")[0] for p in run("prelude")], ["`b.nope`:"])

    def test_std_signatures(self):
        files = {"std/a.cp": "pub fn f<K>(x: K) -> ?K = None\npub fn g() -> Int = 1\n",
                 "spec/lib/a.md": "```cp\nfn f(x: Int) -> ?Int\nfn g() -> Int\n```\nfn f() -> Bool outside a block\n"}
        with Tree(files):
            self.assertEqual([p.where for p in run("std-signatures")], ["spec/lib/a.md:2"])

    def test_case_bugs(self):
        log = ("  core-09/x.cp: fail: wanted trap R0001; got: error[R0001]: ln(-1)\n"
               "  a/b.cp: fail: wanted ok; got: error[E0606]: `xs.min()` has type `?Int`, which does not implement `Show`\n"
               "  a/c.cp: fail: wanted ok; got: error[E0203]: a `?Range<Int>` has no member `len`\n")
        with tempfile.NamedTemporaryFile("w", suffix=".log", delete=False) as f:
            f.write(log)
        try:
            self.assertEqual([(p.where, p.what.split(":")[0]) for p in run("case-bugs", "--log", f.name)],
                             [("core-09/x.cp", "same-code"), ("a/b.cp", "optional")])
        finally:
            os.unlink(f.name)

    def test_sigs(self):
        files = {"std/a.cp": "pub fn f(x: Int; by: Int = 0) -> Int = x\n",
                 "spec/lib/a.md": "```cp\nf(1, by: 2)\nf(1, key: 2)\ng(1, key: 2)\n```\n"}
        with Tree(files):
            self.assertEqual([(p.where, p.what) for p in run("sigs")], [("lib/a.md", "f(key:)")])

    def test_metamorphic(self):
        from specgate import metamorphic
        files = {"research/scoreboard.json": '{"suites": {"conformance": {"cases": {"a.cp": "pass", "b.cp": "pass", "c.cp": "fail"}}}}',
                 "spec/conformance/a.cp": '// expect: ok, output "1"\nprint(1)\n',
                 "spec/conformance/b.cp": '// expect: ok, output "2"\n// input: "x"\nprint(2)\n'}
        def fake(src):
            return (0, "1\n", "") if "\r" not in src else (0, "9\n", "")
        with Tree(files), mock.patch.object(metamorphic, "run_src", fake), \
                mock.patch.dict(metamorphic.TRANSFORMS, {"fmt": lambda s: s}):
            self.assertEqual([(p.where, p.what.split(":")[0]) for p in run("metamorphic", "--jobs", "1")], [("a.cp", "crlf")])

    def test_bugcatch(self):
        from specgate import bugcatch as bc
        with tempfile.TemporaryDirectory() as d:
            root = Path(d)
            (root / "tree").mkdir()

            def fake(cmd, cwd=None, **_):
                if "--scoreboard" in cmd:
                    board = Path(cmd[cmd.index("--scoreboard") + 1])
                    (Path(cwd) / board).write_text(json.dumps({"suites": {"conformance": {"cases": {"a.cp": "pass", "b.cp": "pass"}}}}))
                return mock.Mock(returncode=0)

            old = Path.cwd()
            os.chdir(root)
            try:
                with mock.patch.object(bc.subprocess, "run", fake):
                    cases = bc.run_suite(root / "tree", {"CARGO_TARGET_DIR": "t"}, Path("base.json"))
            finally:
                os.chdir(old)
            self.assertEqual(cases, {"a.cp": "pass", "b.cp": "pass"})
            self.assertTrue((root / "base.json").is_file())
            with mock.patch.object(bc, "known_gaps", lambda: {"b.cp"}):
                self.assertEqual(bc.broken(cases, {"a.cp": "fail", "b.cp": "fail"}), ["a.cp"])

    def test_mutants(self):
        from specgate import mutants
        with Tree({"crates/cpl-check/src/a.rs": "fn f(x: i32) -> bool { x < 1 && true }\n"}) as t:
            pool = mutants.sites()
            self.assertTrue(pool)
            path, i, k = pool[0]
            was, now = mutants.mutate(t.root, path, i, k)
            self.assertNotEqual(was, now)
            self.assertEqual(t.read(path).split("\n")[i], now)

    def test_explain_copies(self):
        from specgate import explain_copies as ec
        for cpp in ("const std::vector<ll>& a = std::get<1>(t0);", "std::vector<ll> t14 = std::vector<ll>();",
                    "std::vector<ll> best((size_t)(12), big);", "std::vector<ll> t1 = std::get<1>(l)(s);",
                    "const std::vector<ll>* src = nullptr;",
                    "struct N {\n  std::vector<ll> big;\n};\ninline const N& get(const A& a, ll i) {\n  return a.xs[i];\n}\nvoid f(const A& a) {\n  const N& n = get(a, 0);\n}",
                    "struct N {\n  std::vector<ll> big;\n};\nN make(ll i) {\n  return N{};\n}\nvoid f() {\n  N n = make(0);\n}",
                    "void f() {\n  std::vector<ll> t14 = g();\n}\nvoid h() {\n  bool t14 = false;\n  t14 = true;\n}",
                    "struct V {\n  ll mn1;\n};\nstruct B {\n  std::vector<ll> mn1;\n};\nvoid f(V& v, const V& p) {\n  v.mn1 = p.mn1;\n}",
                    "static auto cpl_fn0 = [](const std::tuple<int, int>& _p0) mutable -> std::vector<std::tuple<int, ll>> {"):
            self.assertEqual(ec.deep_copies(cpp + "\n"), 0, cpp)
        for cpp in ("std::vector<cpl::u8> w = std::get<1>(top);", "self.ys[i] = f(cpl::sorted<true>(self.ys[i]));", "std::string x = f(r) ? bag[g(r)] : w;", "return std::optional<std::vector<ll>>(std::get<0>(cpl::have(res)));",
                    "struct M {\n  std::vector<ll> xs;\n};\nM f(const M& a) {\n  return g(n == 0 ? M(a) : h(a));\n}",
                    "struct N {\n  std::vector<ll> big;\n};\ninline const N& get(const A& a, ll i) {\n  return a.xs[i];\n}\nvoid f(const A& a) {\n  N n = get(a, 0);\n}",
                    "struct N {\n  std::vector<ll> big;\n};\ninline const N& get(const A& a, ll i) {\n  return a.xs[i];\n}\nvoid f(const A& a) {\n  N n;\n  n = get(a, 0);\n}", "t16[v] = cpl::sorted<true>(sadj[(ll)v]);",
                    "void f(const std::vector<ll>& xs) {\n  std::vector<ll> ys = g();\n  ys = xs;\n}",
                    "struct H {\n  std::vector<ll> xs;\n};\nH f(const std::vector<ll>& xs) {\n  H h_2 = H_new();\n  h_2.xs = xs;\n}",
                    "struct M {\n  std::vector<std::string> vals;\n};\ninline void set(M& self, ll t0, const std::string& v) {\n  self.vals[t0] = v;\n}",
                    "struct C {\n  std::array<char, 7> take;\n};\ninline void rebuild(C& self, const std::array<char, 7>& take) {\n  self.take = take;\n}",
                    "struct B {\n  cpl::Row<cpl::u64, 1> w;\n};\nvoid f(std::vector<B>& out, const B& b) {\n  out[0] = b;\n}",
                    "void f() {\n  auto g = [&c_3 = c_2](const std::vector<ll>& m) mutable -> void {\n    c_3 = m;\n  };\n}",
                    "void f(std::vector<std::string>& out) {\n  static std::vector<std::string> t2;\n  auto& r = t2;\n  out[0] = r[c[0]];\n}",
                    "void f(const std::vector<std::vector<int>>& adj) {\n  auto g = [&adj](int u) mutable -> std::vector<int> {\n    return adj[u];\n  };\n}",
                    "struct M {\n  std::vector<std::string> keys;\n};\nll ins(M& self, const std::string& x) {\n  return [&](auto&& self, auto&& x) -> ll {\n    self.keys[0] = x;\n  }(self, x);\n}",
                    "struct cpl_cl5 {\n  mutable std::vector<ll> c0;\n};\nvoid f() {\n  std::vector<ll> ew = g();\n  h(cpl_cl5{ew});\n}"):
            self.assertEqual(ec.deep_copies(cpp + "\n"), 1, cpp)
        self.assertEqual(ec.deep_copies("auto b = std::vector<int64_t>(a);\n"), 1)
        self.assertEqual(ec.deep_copies("inline T f(const T& self) {\n  return std::tuple<std::string, ll>(self);\n}\n"), 1)
        self.assertEqual(ec.deep_copies("return std::tuple<ll, ll>(self);\nreturn std::tuple<std::string, ll>(f(x));\n"), 0)
        self.assertEqual(ec.deep_copies("std::array<char, 40> a = want[i];\nconst std::array<char, 40>& b = want[j];\n"), 1)
        for line in ("a2 = std::array<ll, 4>(a);", "u = std::set<ll>(a);", "a = cpl::sorted(a);",
                     "g = std::vector<std::vector<ll>>(rows[i]);", "m = std::map<ll, ll>(cpl::at(ms, i));"):
            self.assertEqual(ec.deep_copies(line + "\n"), 1, line)
        holder = "struct H {\n  std::vector<ll> pre;\n};\n"
        for line in ("std::optional<std::string> t0 = best;", "std::optional<H> t0 = of;", "f(std::optional<H>(h2));",
                     "f(std::optional<std::vector<ll>>(xs));", "return std::optional<std::vector<char>>(std::get<0>(sp));",
                     "return std::optional<std::array<ll, 5>>(std::get<1>(r));", "f(std::optional<std::array<ll, 2>>(a));",
                     "auto s = cpl::to_str(cpl::str_slice(s, 0, q));",
                     "inline std::string all(const B& self) {\n  return self.agg[self.bag.root];\n}",
                     "inline H blank(const T& self) {\n  return self.m.id;\n}",
                     "H v = p == 0 ? f(self) : self.t[p];",
                     "std::array<ll, 7> wt = std::move(t);\nauto f = [wt_2 = wt](int e) mutable -> ll { return wt_2[e]; };"):
            self.assertEqual(ec.deep_copies(holder + line + "\n"), 1, line)
        for line in ("std::optional<ll> t0 = best;", "return std::optional<ll>(std::get<0>(sp));",
                     "inline ll at(const T& self) {\n  return self.t[0];\n}",
                     "inline H make(const T& self) {\n  return acc;\n}",
                     "inline std::string& at(T& self) {\n  return self.vals[0];\n}",
                     "inline H make(const T& s) {\n  auto f = [&](ll i) -> ll {\n    return s.t[i];\n  };\n}",
                     "std::add_pointer_t<std::vector<ll>(ll)> leaf;",
                     "std::array<ll, 7> wt = std::move(t);\nauto f = [&wt_2 = wt](int e) -> ll { return wt_2[e]; };",
                     "namespace cpl {\nstd::string to_str() const { return std::string(sv); }\n}  // namespace cpl"):
            self.assertEqual(ec.deep_copies(holder + line + "\n"), 0, line)
        for line in ("auto v = std::vector<ll>(5);", "auto t = cpl::sorted(f(a));", "auto v = std::vector<ll>(s.begin(), s.end());"):
            self.assertEqual(ec.deep_copies(line + "\n"), 0, line)
        rows = {"a.cp": ("a.cp", 2, 1), "b.cp": ("b.cp", 1, 1), "c.cp": ("c.cp", 0, 2)}
        with Tree({"spec/conformance/a.cp": "", "spec/conformance/b.cp": "", "spec/conformance/c.cp": ""}), \
                mock.patch.object(ec, "one", lambda cpl, c, tmp, exact: rows.get(c.name)), \
                mock.patch.object(ec, "default_cache_dir", lambda: None):
            over = ("a.cp", "explain over-lists copies: it lists 2, the C++ makes 1")
            under = ("c.cp", "explain under-lists copies: it lists 0, the C++ makes 2")
            self.assertEqual([(p.where, p.what) for p in run("explain-copies")], [over])
            self.assertEqual(run("explain-copies", "--max", "1"), [])
            self.assertEqual([(p.where, p.what) for p in run("explain-copies", "--exact", "--max", "1")], [over, under])

    def test_explain_copies_cache_matches_a_cold_run_and_an_edit_recomputes_only_its_case(self):
        from specgate import explain_copies as ec
        calls = []

        def one(cpl, case, tmp, exact):
            calls.append(case.name)
            text = case.read_text()
            return None if not text else (case.relative_to(specgate.ROOT), text.count("c"), 1)

        files = {f"spec/conformance/{n}.cp": t for n, t in (("a", "cc"), ("b", "c"), ("c", "ccc"), ("d", ""))}
        files |= {"std/core.cp": "pub fn f() -> Int = 1\n", "target/debug/cpl": "v1"}
        with Tree(files) as t, mock.patch.object(ec, "one", one):
            cache = str(t.root / "cache")

            def gate(*more):
                calls.clear()
                got = [(p.where, p.what) for p in run("explain-copies", "--exact", "--max", "0", *more)]
                return got, sorted(calls)

            uncached, _ = gate("--no-cache")
            self.assertEqual(gate("--cache-dir", cache), (uncached, ["a.cp", "b.cp", "c.cp", "d.cp"]))
            self.assertEqual(gate("--cache-dir", cache), (uncached, []))
            (t.root / "spec/conformance/b.cp").write_text("cc")
            edited, _ = gate("--no-cache")
            self.assertNotEqual(edited, uncached)
            self.assertEqual(gate("--cache-dir", cache), (edited, ["b.cp"]))
            for changed in ("std/core.cp", "target/debug/cpl"):
                (t.root / changed).write_text("changed")
                self.assertEqual(gate("--cache-dir", cache), (edited, ["a.cp", "b.cp", "c.cp", "d.cp"]))

    def test_decisions(self):
        # reserve and --check-landing run against a scratch remote in test_spec_gate_decisions.py
        with Tree({"decisions.md": "## 1. A\n## 2. B\n## 2. C\n"}) as t:
            self.assertEqual([p.what for p in run("decisions", "--check", "--file", str(t.root / "decisions.md"))],
                             ["decision number 2 is used twice"])
            (t.root / "decisions.md").write_text("## 1. A\n## 2. B\n")
            self.assertEqual(run("decisions", "--check", "--file", str(t.root / "decisions.md")), [])

    def test_std_docs(self):
        doc = "/// Adds.\n/// cost: O(1)\n/// traps: never\n/// example: f(1) == 1\n"
        files = {"std/a.cp": doc + "pub fn f(x: Int) -> Int = x\npub fn k<A: Ord<A> > (\n    x: A,\n) -> Int = 1\nstruct S(n: Int):\n    /// cost: O(n)\n    @get pub fn g(self) -> Int = 1\n",
                 "std/prelude.cp": "use std.a.{f, h}\n",
                 "spec/core/a.md": "`f(x)`\n"}
        with Tree(files) as t:
            got = [(p.where, p.what.split(" ")[0]) for p in run("std-docs")]
            self.assertIn(("std/prelude.cp", "`h`"), got)
            self.assertIn(("std/a.cp:11", "`g`"), got)
            self.assertEqual([p for p in run("std-docs") if "return type" in p.what], [])
            self.assertEqual([p.where for p in run("std-docs", "--audit", str(t.root / "std/a.cp"))], ["std/a.cp:6", "std/a.cp:11"])
            self.assertEqual(run("std-docs", "--stub", str(t.root / "std/a.cp")), [])
            self.assertIn("todo", t.read("std/a.cp"))

    def test_tool_vocab(self):
        from specgate import tool_vocab as tv
        self.assertEqual(tv.prose("a `borrow` b"), "a   b")

        class Out:
            def __init__(self, out):
                self.stdout, self.stderr, self.returncode = out, "", 0

        class Lsp:
            def __init__(self, cpl):
                self.p = mock.Mock()

            def hovers(self, path, src):
                yield "an inout param"

        glossary = ("| Term | Meaning | Never |\n|---|---|---|\n"
                    "| piece | p | segment (except in segment trees), interval (except the `interval` module) |\n"
                    "| position | p | index (as a type name), node id |\n"
                    "| `None` | v | `none`, `nil` |\n"
                    "| trap | t | panic |\n"
                    "| `mut` | m | `inout` |\n")
        rules = tv.banned(glossary, spec=tv.prose("none of the cases; an index"))
        self.assertEqual([s for _, s, _, _ in rules], ["segment", "interval", "node id", "`nil`", "panic", "`inout`"])
        hits = lambda text: [s for p, s, _, _ in rules if p.search(tv.prose(text))]
        self.assertEqual(hits("a segment tree, a `segment`, it panicked, Node IDs, none, nil"), ["node id", "`nil`", "panic"])
        self.assertEqual(hits("two segments"), ["segment"])
        self.assertEqual([r[-1] for r in rules if r[-1]], [{"interval"}])

        problem = pathlib.PurePath("corpus/problems").joinpath("p1")
        port = str(problem / "port.cp")
        files = {port: "print(2)\n", str(problem / "notes.md"): "a segment\n", "std/interval.cp": "/// merged intervals\nfn f() {}\n", "std/window.cp": "// an interval\n/// a movable interval\n", "spec/conformance/a.cp": "print(1)\n", "spec/conformance/b.cp": "// expect: E0302\nlet x: Int = 1.5\n",
                 "spec/GLOSSARY.md": glossary,
                 "crates/cpl-diag/src/msgs/check.rs": 'super::msgs! {\n    A = TYPE_MISMATCH, "a `{}` panics";\n    B = "L0001", "an inout `{}`";\n}\n'}

        def sh(cmd, **_):
            if cmd[1:] == ["--help"]:
                return Out("Usage\n  run   runs it\n")
            if cmd[1:] == ["explain", "--json"]:
                return Out(json.dumps([{"code": "E0001", "title": "a node id", "summary": "`inout`"}]))
            if cmd[1:3] == ["check", "--json"]:
                if cmd[3].endswith("port.cp"):
                    return Out("")
                d = {"code": "E0302", "start": {"line": 2}, "message": "a nil here", "fixes": [{"text": "nil"}]}
                return Out(json.dumps({"files": [{"diagnostics": [d]}]}))
            return Out("this panics")

        with Tree(files) as tree, mock.patch.object(tv, "sh", sh), mock.patch.object(tv, "Lsp", Lsp), \
                mock.patch.object(corpus, "PROBLEMS", str(tree.root.joinpath("corpus/problems"))):
            got = sorted(p.where for p in run("tool-vocab"))
        self.assertEqual(got, [f"cpl check {port}", "cpl check spec/conformance/b.cp:2 E0302",
                               "cpl explain E0001 (title)", "cpl run --help", "crates/cpl-diag/src/msgs/check.rs:2 A",
                               "crates/cpl-diag/src/msgs/check.rs:3 B", "hover in spec/conformance/a.cp", "std/window.cp:2"])

    def test_coverage(self):
        import importlib
        from specgate import coverage as cov
        cov = importlib.reload(cov)
        m = cov.RULE_DEF.search("is legal and adds a flag. (R02.4.3, permits)")
        self.assertEqual((m.group(1), m.group(2)), ("02.4.3", ", permits"))
        self.assertIsNone(cov.RULE_DEF.search("rejects it. (R02.4.4)").group(2))
        self.assertIn("01.3.6", cov.rule_texts())
        self.assertIn("01.3.6", cov.PERMITS)
        self.assertNotIn("01.3.5", cov.PERMITS)
        self.assertFalse((specgate.ROOT / "spec/conformance/permits.txt").exists())

    def test_coverage_writes_its_report_outside_the_tree_or_prints_it(self):
        import argparse
        import contextlib
        import io
        from specgate import coverage as cov
        with tempfile.TemporaryDirectory() as tmp, mock.patch.object(cov, "REPORT", pathlib.Path(tmp) / "spec/COVERAGE.md"):
            self.assertEqual(cov.run(argparse.Namespace(print=False)), [])
            report = cov.REPORT.read_text()
            self.assertTrue(report.startswith("# Conformance coverage\n"))
            out = io.StringIO()
            with contextlib.redirect_stdout(out):
                self.assertEqual(cov.run(argparse.Namespace(print=True)), [])
            self.assertEqual(out.getvalue(), report)
        self.assertFalse((specgate.ROOT / "spec/conformance/COVERAGE.md").exists())

    def test_markers(self):
        old = "// expect: E0601 at 2 (checked)\n// rule: 05.1\n// a note\nlet a = 1\r\nf(a)\n"
        with Tree({"spec/conformance/c/a.cp": old}) as t:
            self.assertEqual(len(run("markers")), 1)
            self.assertEqual(run("markers", "--fix"), [])
            self.assertEqual(t.read("spec/conformance/c/a.cp"),
                             "// expect: E0601 (checked)\n// rule: 05.1\n// a note\nlet a = 1 // <- here\r\nf(a)\n")
            self.assertEqual(run("markers"), [])

    def test_subset(self):
        from specgate import subset
        files = {"spec/core/01-x.md": "# 1. Lexing\n\n## 1.5 Literals\n\nA rule (R01.5.1).\n",
                 "spec/core/02-y.md": "## 2.1 Types\n\nA rule (R02.1.1).\n\n## 2.2 Casts\n\nAs in (R01.6.1).\n",
                 "spec/core/03-z.md": "## 3.6 Ints\n\nA rule (R01.6.1).\n",
                 "spec/conformance/c/f.cp": "// expect: ok\n// rule: 01.5.1\nx\n",
                 "research/scoreboard.json": '{"suites": {"conformance": {"cases": {"c/f.cp": "fail"}}}}',
                 "spec/SUBSET.md": "| R02.1 | 2.1 Types | documented | later |\n"}
        with Tree(files) as t, mock.patch.object(subset, "SUBSET", t.root / "spec/SUBSET.md"), \
                mock.patch.object(subset, "SPEC", t.root / "spec"):
            self.assertEqual([p.what for p in run("subset")][-1][:8], "is stale")
            self.assertEqual(run("subset", "--write"), [])
            text = t.read("spec/SUBSET.md")
            self.assertIn("| R01.5 | 1.5 Literals | supported | 1 known gap |", text)
            self.assertIn("| R02.1 | 2.1 Types | documented | later |", text)
            self.assertNotIn("R01.6 | 2.2 Casts", text)
            self.assertIn("- c/f.cp: fail", text)
            self.assertEqual(run("subset"), [])


if __name__ == "__main__":
    unittest.main()
