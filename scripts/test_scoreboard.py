#!/usr/bin/env python3
"""Unit tests for scripts/scoreboard.py.

    python3 scripts/test_scoreboard.py
"""
import json
import os
import subprocess
import sys
import tempfile
import unittest

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
import scoreboard as sb


class TokenizersTest(unittest.TestCase):
    def test_cp_tokenizer_strips_comments_and_handles_multi_char_ops(self):
        src = """
        // header comment
        let x: 0..=10 = read()
        /* block
           comment */
        var y = x ?? 0
        y += 1
        print("hello // not comment")
        """
        toks = sb.tokenize_cp(src)
        self.assertIn("..=", toks)
        self.assertIn("??", toks)
        self.assertIn("+=", toks)
        self.assertIn('"hello // not comment"', toks)
        self.assertNotIn("header", toks)
        self.assertNotIn("block", toks)

    def test_cpp_tokenizer_strips_comments_and_counts_ops(self):
        src = """
        // comment
        int main() {
            /* block */
            int x = 0;
            x += 1;
            std::cout << x << "\\n";
        }
        """
        toks = sb.tokenize_cpp(src)
        self.assertIn("::", toks)
        self.assertIn("<<", toks)
        self.assertIn("+=", toks)
        self.assertNotIn("comment", toks)
        self.assertNotIn("block", toks)

    def test_strip_cpp_library_reduces_template_boilerplate(self):
        with tempfile.TemporaryDirectory() as d:
            p = os.path.join(d, "foo_orig.cpp")
            with open(p, "w") as f:
                f.write(
                    "#include <bits/stdc++.h>\n"
                    "using namespace std;\n"
                    "template <typename T> struct SegTree { int n; };\n"
                    "int main() {\n"
                    "    int n; cin >> n;\n"
                    "    cout << n + 1 << '\\n';\n"
                    "}\n"
                )
            with open(p, encoding="utf-8") as f:
                raw = len(sb.tokenize_cpp(f.read()))
            stripped = len(sb.tokenize_cpp(sb.strip_cpp_library(p)))
            self.assertLess(stripped, raw)
            self.assertGreater(stripped, 0)


class FrictionSummaryTest(unittest.TestCase):
    def test_parses_sections_skips_measurements_and_counts_severities(self):
        with tempfile.TemporaryDirectory() as d:
            with open(os.path.join(d, "README.md"), "w") as f:
                f.write("# README\n## Template\n- **Severity:** blocker\n")
            with open(os.path.join(d, "sample.md"), "w") as f:
                f.write(
                    "# sample\n\n"
                    "## Missing ODT\n"
                    "- **Where:** lines 1-5\n"
                    "- **What felt wrong:** manual split\n"
                    "- **Goal it hurts:** express, hard-to-get-wrong.\n"
                    "- **Severity:** high.\n\n"
                    "## Cannot express feature X\n"
                    "- **Goal it hurts:** express\n"
                    "- **Severity:** blocker (could not port)\n\n"
                    "## ~~Old resolved bug~~\n"
                    "- **Goal it hurts:** performance\n"
                    "- **Severity:** medium\n\n"
                    "## Measurements\n"
                    "- **Contestant-written tokens:** 100 vs 200\n"
                )
            fr = sb.summarize_frictions(d)
            self.assertEqual(fr["files"], 1)
            self.assertEqual(fr["total"], 3)
            self.assertEqual(fr["open"], 2)
            self.assertEqual(fr["closed"], 1)
            self.assertEqual(fr["by_severity"]["blocker"], 1)
            self.assertEqual(fr["by_severity"]["high"], 1)
            self.assertEqual(fr["by_severity"]["medium"], 0)
            self.assertEqual(fr["by_goal"]["express"], 2)
            self.assertEqual(fr["by_goal"]["hard-to-get-wrong"], 1)
            self.assertEqual(len(fr["open_blockers"]), 1)
            self.assertEqual(fr["open_blockers"][0]["title"], "Cannot express feature X")
            self.assertEqual(fr["warnings"], [])

    def test_parses_h3_frictions_and_skips_meta_sections(self):
        with tempfile.TemporaryDirectory() as d:
            with open(os.path.join(d, "h3.md"), "w", encoding="utf-8") as f:
                f.write(
                    "# h3\n\n"
                    "## Algorithm summary\nSome prose.\n\n"
                    "## Frictions\n\n"
                    "### 1. Missing cyclic convolution\n"
                    "**Goal it hurts:** express, performance\n"
                    "**Severity:** high\n\n"
                    "## What worked well\nEverything else.\n"
                )
            fr = sb.summarize_frictions(d)
            self.assertEqual(fr["total"], 1)
            self.assertEqual(fr["by_severity"]["high"], 1)
            self.assertEqual(fr["by_goal"]["express"], 1)
            self.assertEqual(fr["warnings"], [])


class BaselineDiffTest(unittest.TestCase):
    def _make_report(self, gate="pass", emitted=10000, tok_ratio=0.45, wa_trap=0, blockers=0, time_p50=1.0):
        return {
            "totals": {
                "ports_total": 1,
                "ports_passed": 1 if gate == "pass" else 0,
                "ports_failed": 0 if gate == "pass" else 1,
                "tokens": {
                    "median_ratio": tok_ratio,
                    "median_stripped_ratio": tok_ratio,
                },
                "emitted_bytes": {"max": emitted},
                "perf": {"time_p50": time_p50, "time_p95": time_p50, "rss_p50": 1.0, "rss_p95": 1.0},
                "friction_lint": {"wa_trap": wa_trap},
                "frictions": {"open_blockers": blockers},
            },
            "ports": [
                {
                    "id": "ecnerwala/p1",
                    "gate": gate,
                    "gate_detail": "ok" if gate == "pass" else "check failed",
                    "emitted_bytes": emitted,
                    "cp_sha": "aaa",
                    "orig_sha": "bbb",
                    "perf": {"measured": True, "output_match": True, "time_ratio": time_p50},
                }
            ],
        }

    def test_clean_diff_has_no_regressions(self):
        base = self._make_report()
        cur = self._make_report()
        d = sb.diff_against_baseline(cur, base, mode="fast")
        self.assertEqual(d["regressions"], [])

    def test_detects_gate_emitted_token_blocker_and_perf_regressions(self):
        base = self._make_report(gate="pass", emitted=10000, tok_ratio=0.45, wa_trap=0, blockers=0, time_p50=1.0)
        cur = self._make_report(gate="fail", emitted=12000, tok_ratio=0.50, wa_trap=1, blockers=1, time_p50=1.25)
        d = sb.diff_against_baseline(cur, base, mode="full")
        joined = "\n".join(d["regressions"])
        self.assertIn("gate regression", joined)
        self.assertIn("emitted C++ grew >5%", joined)
        self.assertIn("median raw token ratio regressed", joined)
        self.assertIn("friction_lint wa_trap > 0", joined)
        self.assertIn("open blocker frictions increased", joined)
        self.assertIn("runtime p50 regressed >10%", joined)

    def test_merge_baseline_perf_preserves_unchanged_ports(self):
        base = self._make_report(time_p50=0.92)
        cur = self._make_report()
        cur["ports"][0]["perf"] = None
        sb.merge_baseline_perf(cur["ports"], base)
        self.assertEqual(cur["ports"][0]["perf"]["time_ratio"], 0.92)


class OutputsMatchTest(unittest.TestCase):
    def test_exact_and_float_epsilon_matching(self):
        with tempfile.TemporaryDirectory() as d:
            a = os.path.join(d, "a")
            b = os.path.join(d, "b")
            with open(a, "w", encoding="utf-8") as f:
                f.write("1 2.000001\n3\n")
            with open(b, "w", encoding="utf-8") as f:
                f.write("1   2.000000\n  3 ")
            self.assertFalse(sb.outputs_match(a, b, eps=None))
            self.assertTrue(sb.outputs_match(a, b, eps=1e-5))
            self.assertFalse(sb.outputs_match(a, b, eps=1e-9))


class CheckerHookTest(unittest.TestCase):
    def test_run_checker_accepts_valid_alternative_and_rejects_wrong(self):
        with tempfile.TemporaryDirectory() as d:
            chk = os.path.join(d, "sum_checker.py")
            with open(chk, "w", encoding="utf-8") as f:
                f.write(
                    "import sys\n"
                    "target = int(open(sys.argv[1]).read().strip())\n"
                    "out = [int(x) for x in open(sys.argv[2]).read().split()]\n"
                    "ans = [int(x) for x in open(sys.argv[3]).read().split()]\n"
                    "if sum(out) != target or len(out) != len(ans):\n"
                    "    sys.stderr.write(f'expected sum {target}, got {sum(out)}\\n')\n"
                    "    sys.exit(1)\n"
                )
            inp = os.path.join(d, "in.txt")
            ans = os.path.join(d, "ans.txt")
            alt = os.path.join(d, "alt.txt")
            bad = os.path.join(d, "bad.txt")
            for path, content in [(inp, "10\n"), (ans, "3 7\n"), (alt, "4 6\n"), (bad, "4 5\n")]:
                with open(path, "w", encoding="utf-8") as f:
                    f.write(content)
            ok, err = sb.run_checker(chk, inp, alt, ans)
            self.assertTrue(ok, err)
            ok_bad, err_bad = sb.run_checker(chk, inp, bad, ans)
            self.assertFalse(ok_bad)
            self.assertIn("expected sum 10, got 9", err_bad)

    def test_discover_ports_detects_checker_and_totals_counts_it(self):
        with tempfile.TemporaryDirectory() as d:
            os.makedirs(os.path.join(d, "checkers"))
            with open(os.path.join(d, "p1.cp"), "w", encoding="utf-8") as f:
                f.write("print(1)\n")
            with open(os.path.join(d, "p1_orig.cpp"), "w", encoding="utf-8") as f:
                f.write("int main() {}\n")
            with open(os.path.join(d, "checkers", "p1.py"), "w", encoding="utf-8") as f:
                f.write("import sys; sys.exit(0)\n")
            ports = sb.discover_ports(port_dirs=[d])
            self.assertEqual(len(ports), 1)
            self.assertIsNotNone(ports[0]["checker"])
            totals = sb.compute_totals(
                [
                    {
                        "id": "t/p1",
                        "gate": "pass",
                        "cp_tokens": 5,
                        "orig_tokens": 10,
                        "orig_stripped_tokens": 10,
                        "token_ratio": 0.5,
                        "stripped_token_ratio": 0.5,
                        "emitted_bytes": 1000,
                        "interactive": False,
                        "has_checker": True,
                    }
                ],
                {"files": 1, "wa_trap": 0, "real_gap": 0, "stale_port": 0},
                {
                    "files": 0,
                    "total": 0,
                    "open": 0,
                    "closed": 0,
                    "by_severity": {},
                    "by_goal": {},
                    "open_blockers": [],
                    "open_highs": [],
                },
            )
            self.assertEqual(totals["checker_ports"], 1)
            self.assertEqual(totals["interactive_ports"], 0)


class DreamAndPerfRatchetTest(unittest.TestCase):
    def test_dream_discovery_and_totals(self):
        with tempfile.TemporaryDirectory() as d:
            os.makedirs(os.path.join(d, "dream"))
            with open(os.path.join(d, "p1.cp"), "w", encoding="utf-8") as f:
                f.write("let n: Int = read()\nprint(n + 1)\n")
            with open(os.path.join(d, "p1_orig.cpp"), "w", encoding="utf-8") as f:
                f.write("int main() { int n; std::cin >> n; std::cout << n + 1; }\n")
            with open(os.path.join(d, "dream", "p1.cp"), "w", encoding="utf-8") as f:
                f.write("print(read<Int>() + 1)\n")
            ports = sb.discover_ports(port_dirs=[d])
            self.assertEqual(len(ports), 1)
            self.assertIsNotNone(ports[0]["dream"])
            tok = sb.measure_tokens(ports[0])
            self.assertIsNotNone(tok["dream_tokens"])
            self.assertGreater(tok["cp_tokens"], tok["dream_tokens"])
            self.assertGreater(tok["dream_ratio"], 1.0)
            self.assertEqual(tok["dream_gap_tokens"], tok["cp_tokens"] - tok["dream_tokens"])
            totals = sb.compute_totals(
                [
                    {
                        "id": "t/p1",
                        "gate": "pass",
                        "emitted_bytes": 1000,
                        **tok,
                    }
                ],
                {"files": 1, "wa_trap": 0, "real_gap": 0, "stale_port": 0},
                {
                    "files": 0,
                    "total": 0,
                    "open": 0,
                    "closed": 0,
                    "by_severity": {},
                    "by_goal": {},
                    "open_blockers": [],
                    "open_highs": [],
                },
            )
            self.assertEqual(totals["dream"]["ports"], 1)
            self.assertEqual(totals["dream"]["dream_total"], tok["dream_tokens"])
            self.assertEqual(totals["dream"]["max_port"], "t/p1")

    def test_gen_max_flag_preferred(self):
        with tempfile.TemporaryDirectory() as d:
            gen = os.path.join(d, "g.py")
            with open(gen, "w", encoding="utf-8") as f:
                f.write("import sys\nprint('MAX_CASE' if '--max' in sys.argv else 'SMALL')\n")
            p = {"id": "t/g", "name": "g", "gen": gen, "interactor": None, "samples": []}
            inp, src = sb.generate_max_input(p, "/bin/true", d, {})
            self.assertEqual(src, "gen --max")
            self.assertEqual(sb._read_text(inp).strip(), "MAX_CASE")

    def test_spec_without_generator_marks_unmeasured(self):
        with tempfile.TemporaryDirectory() as d:
            gen = os.path.join(d, "g.py")
            with open(gen, "w", encoding="utf-8") as f:
                f.write("print(1)\n")
            p = {"id": "t/g", "name": "g", "gen": gen, "interactor": None, "samples": []}
            spec = ("t/g", None, "x", "limits not on file", None, "exact")
            inp, src = sb.generate_max_input(p, "/bin/true", d, {"t/g": spec})
            self.assertIsNone(inp)
            self.assertEqual(src, "limits not on file")

    def test_per_port_runtime_ratchet_and_worst_10(self):
        def make_rep(ratios):
            ports = [
                {
                    "id": pid,
                    "gate": "pass",
                    "cp_tokens": 10,
                    "orig_tokens": 20,
                    "orig_stripped_tokens": 20,
                    "token_ratio": 0.5,
                    "stripped_token_ratio": 0.5,
                    "emitted_bytes": 1000,
                    "perf": {
                        "measured": True,
                        "output_match": True,
                        "cpp_ms": round(orig_ms * r, 2),
                        "orig_ms": orig_ms,
                        "time_ratio": r,
                        "rss_ratio": 1.0,
                        "input_source": "gen --max",
                    },
                }
                for pid, r, orig_ms in ratios
            ]
            totals = sb.compute_totals(
                ports,
                {"files": len(ports), "wa_trap": 0, "real_gap": 0, "stale_port": 0},
                {
                    "files": 0,
                    "total": 0,
                    "open": 0,
                    "closed": 0,
                    "by_severity": {},
                    "by_goal": {},
                    "open_blockers": [],
                    "open_highs": [],
                },
            )
            return {"ports": ports, "totals": totals}

        base = make_rep([("t/fast", 1.0, 150.0), ("t/fast2", 1.0, 150.0), ("t/tiny", 1.0, 20.0), ("t/slow", 2.0, 300.0)])
        self.assertTrue(base["totals"]["perf"]["p50_ok"])
        self.assertFalse(base["totals"]["perf"]["max_ok"])
        self.assertEqual(base["totals"]["perf"]["over_1_5x"], 1)
        self.assertEqual(base["totals"]["perf"]["worst_10"][0]["id"], "t/slow")
        self.assertNotIn("t/tiny", [w["id"] for w in base["totals"]["perf"]["worst_10"]])
        self.assertEqual(base["totals"]["perf"]["above_floor"], 3)

        # A port whose original runs under 100 ms is not counted: its ratio measures startup
        cur_ok = make_rep([("t/fast", 1.05, 150.0), ("t/fast2", 1.05, 150.0), ("t/tiny", 2.5, 20.0), ("t/slow", 2.05, 300.0)])
        diff_ok = sb.diff_against_baseline(cur_ok, base, mode="full")
        self.assertEqual(diff_ok["regressions"], [])

        cur_reg = make_rep([("t/fast", 1.15, 150.0), ("t/fast2", 1.0, 150.0), ("t/tiny", 1.0, 20.0), ("t/slow", 2.0, 300.0)])
        diff_reg = sb.diff_against_baseline(cur_reg, base, mode="full")
        self.assertTrue(any("runtime regressed >10%: t/fast" in m for m in diff_reg["regressions"]))

        # A new max input (another generator) is a new measurement, not a regression
        cur_new_input = make_rep([("t/fast", 1.6, 150.0), ("t/fast2", 1.0, 150.0), ("t/tiny", 1.0, 20.0), ("t/slow", 2.0, 300.0)])
        cur_new_input["ports"][0]["perf"]["input_source"] = "spec (n=2e5)"
        diff_new_input = sb.diff_against_baseline(cur_new_input, base, mode="full")
        self.assertFalse(any("t/fast " in m for m in diff_new_input["regressions"]))

    def test_dream_gap_ratchet_interp_ratio_ratchet_and_taste_score(self):
        with tempfile.TemporaryDirectory() as d:
            with open(os.path.join(d, "p1.md"), "w", encoding="utf-8") as f:
                f.write("# Review: p1\n\n- **Score:** 4/5\n")
            score, rel = sb.parse_review_score("p1", reviews_dir=d)
            self.assertEqual(score, 4)
            self.assertIsNotNone(rel)
            self.assertEqual(sb.parse_review_score("missing", reviews_dir=d), (None, None))

        def rep(dream_ratio, interp_ratio, taste):
            ports = [
                {
                    "id": "t/p1",
                    "gate": "pass",
                    "cp_tokens": 15,
                    "orig_tokens": 30,
                    "orig_stripped_tokens": 30,
                    "token_ratio": 0.5,
                    "stripped_token_ratio": 0.5,
                    "dream_tokens": 10,
                    "dream_ratio": dream_ratio,
                    "taste_score": taste,
                    "emitted_bytes": 1000,
                    "interp": {
                        "ok": True,
                        "ms": 50.0,
                        "ratio": interp_ratio,
                        "ratio_interp_ms": 50.0,
                        "ratio_cpp_ms": 50.0 / interp_ratio,
                        "ratio_source": "max",
                    },
                }
            ]
            totals = sb.compute_totals(
                ports,
                {"files": 1, "wa_trap": 0, "real_gap": 0, "stale_port": 0},
                {
                    "files": 0,
                    "total": 0,
                    "open": 0,
                    "closed": 0,
                    "by_severity": {},
                    "by_goal": {},
                    "open_blockers": [],
                    "open_highs": [],
                },
            )
            return {"ports": ports, "totals": totals}

        base = rep(1.5, 8.0, 4)
        self.assertEqual(base["totals"]["taste"]["median"], 4)
        self.assertTrue(base["totals"]["interp"]["bar_ok"])
        self.assertFalse(base["totals"]["interp"]["north_star_ok"])

        cur_reg = rep(1.6, 9.5, 4)
        diff_fast = sb.diff_against_baseline(cur_reg, base, mode="fast")
        self.assertTrue(any("dream gap regressed: t/p1" in m for m in diff_fast["regressions"]))
        rewritten = rep(1.6, 8.0, 4)
        rewritten["ports"][0]["dream_tokens"] = 9
        diff_rw = sb.diff_against_baseline(rewritten, base, mode="fast")
        self.assertFalse(any("dream gap regressed" in m for m in diff_rw["regressions"]))
        self.assertTrue(any("dream rewritten: t/p1" in m for m in diff_rw["improvements"]))
        diff_full = sb.diff_against_baseline(cur_reg, base, mode="full")
        self.assertTrue(any("interp ratio regressed >10%: t/p1" in m for m in diff_full["regressions"]))
        self.assertTrue(any("interp ratio p50 regressed" in m for m in diff_full["regressions"]))

    def test_friction_area_routing_and_metadata_warning_gate(self):
        with tempfile.TemporaryDirectory() as d:
            p = os.path.join(d, "bad.md")
            with open(p, "w", encoding="utf-8") as f:
                f.write("# bad\n\n## SegTree lacks lazy push\n- **Severity:** high\n- **Goal it hurts:** performance\n\n## Missing meta\nno fields here\n")
            summary = sb.summarize_frictions(d)
            self.assertEqual(summary["by_area"]["ranges"], 1)
            self.assertEqual(len(summary["warnings"]), 2)
            base = {"ports": [], "totals": {"frictions": {"open_blockers": 0, "warnings": 0}, "tokens": {}, "friction_lint": {"wa_trap": 0}}}
            cur = {"ports": [], "totals": {"frictions": {"open_blockers": 0, "warnings": 2}, "tokens": {}, "friction_lint": {"wa_trap": 0}}}
            diff = sb.diff_against_baseline(cur, base, mode="fast")
            self.assertTrue(any("friction metadata warnings grew" in m for m in diff["regressions"]))

    def test_friction_explicit_area_beats_keywords(self):
        # Explicit decorated Area: fields must take precedence over body keywords like "point" or "tree".
        cases = [
            ("Two-pointer cyclic span", "uses two-pointer wraparound", {"area": "seq (`plan/STDLIB.md`)"}, "seq"),
            ("Number theory choose", "points on a grid", {"area": "number theory (`plan/STDLIB.md`) + core language"}, "math"),
            ("Rollback DSU checkpoint", "checkpoint on graph", {"area": "containers (`plan/STDLIB.md`) + codegen"}, "containers"),
            ("Backticked strings", "point-wise string", {"area": "`strings` (`plan/STDLIB.md` 4.13)"}, "strings"),
            ("Codegen pass", "convex hull helper", {"area": "codegen"}, "core"),
            ("Core I/O", "polygon input", {"area": "core language / I/O"}, "core"),
            ("Geometry hull", "tree-like search", {"area": "geometry (`plan/STDLIB.md`)"}, "geometry"),
            ("Monotone set", "prefix maxima, convex hull trick, staircase", {"area": "containers (`plan/STDLIB.md`)"}, "containers"),
            # When Area: is missing, fall back to keywords without matching "pointer"/"midpoint" as geometry.
            ("Fixed-point walk", "no area field, uses midpoint and two-pointer", {}, "core"),
            ("Convex hull helper", "no area field, convex hull over 2D points", {}, "geometry"),
        ]
        for title, body, fields, want in cases:
            with self.subTest(title=title, fields=fields):
                self.assertEqual(sb.route_friction_area(title, body, fields), want)


class CliSmokeTest(unittest.TestCase):
    def test_frictions_cli_and_fast_subset_cli(self):
        tool = os.path.join(os.path.dirname(os.path.abspath(__file__)), "scoreboard.py")
        r = subprocess.run([sys.executable, tool, "--frictions", "--json"], capture_output=True, text=True)
        self.assertEqual(r.returncode, 0, r.stderr)
        data = json.loads(r.stdout)
        self.assertIn("by_severity", data)
        self.assertIn("by_goal", data)


class TasteCountsTest(unittest.TestCase):
    def test_reads_per_file_lines_and_skips_the_total(self):
        with tempfile.TemporaryDirectory() as d:
            fake = os.path.join(d, "cpl")
            with open(fake, "w") as f:
                f.write(
                    "#!/bin/sh\n"
                    "echo '   3  /p/a.cp  L0503:1 L0504:2'\n"
                    "echo '   3  total over 1 file of 2  L0503:1 L0504:2'\n"
                )
            os.chmod(fake, 0o755)
            got = sb.taste_counts(fake, ["/p/a.cp", "/p/b.cp"])
        self.assertEqual(got, {"/p/a.cp": (3, {"L0503": 1, "L0504": 2})})
        self.assertEqual(
            sb.taste_by_lint([{"taste_lints_by_lint": {"L0503": 1}}, {"taste_lints_by_lint": {"L0503": 2, "L0501": 1}}]),
            {"L0501": 1, "L0503": 3},
        )


class GoalTestsAndLeaderboardTest(unittest.TestCase):
    def test_bug_replay_suite_live_and_ratchet(self):
        cpl = sb.find_cpl_binary()
        if not os.path.isfile(cpl):
            self.skipTest("cpl binary not built")
        br = sb.run_bug_replay_suite(cpl)
        self.assertIsNotNone(br)
        self.assertGreaterEqual(br["total"], 60)
        self.assertTrue(br["bar_ok"], br)
        self.assertGreaterEqual(br["handled_pct"], 90.0)
        self.assertEqual(br["unexpected_pass"], 0)
        self.assertGreaterEqual(len(br["by_class"]), 11)
        for cls, info in br["by_class"].items():
            self.assertGreaterEqual(info["total"], 5, f"{cls} has <5 cases: {info}")

        base = {
            "ports": [],
            "totals": {
                "frictions": {"open_blockers": 0, "warnings": 0},
                "tokens": {},
                "friction_lint": {"wa_trap": 0},
                "bug_replay": {
                    "handled_pct": 91.7,
                    "bar_pct": 90.0,
                    "bar_ok": True,
                    "unexpected_pass": 0,
                    "cases": [{"id": "c1", "outcome": "prevented"}],
                },
            },
        }
        cur_regressed = {
            "ports": [],
            "totals": {
                "frictions": {"open_blockers": 0, "warnings": 0},
                "tokens": {},
                "friction_lint": {"wa_trap": 0},
                "bug_replay": {
                    "handled_pct": 83.3,
                    "bar_pct": 90.0,
                    "bar_ok": False,
                    "unexpected_pass": 0,
                    "cases": [{"id": "c1", "outcome": "caught_on_run"}],
                },
            },
        }
        diff = sb.diff_against_baseline(cur_regressed, base, mode="fast")
        self.assertTrue(any("bug-replay handled" in m for m in diff["regressions"]))
        self.assertTrue(any("bug-replay case regressed" in m for m in diff["regressions"]))

    def test_revision_suite_live_and_ratchet(self):
        cpl = sb.find_cpl_binary()
        if not os.path.isfile(cpl):
            self.skipTest("cpl binary not built")
        rv = sb.run_revision_suite(cpl)
        self.assertIsNotNone(rv)
        self.assertGreaterEqual(rv["total"], 10)
        self.assertEqual(rv["passed"], rv["total"], rv)
        self.assertGreater(rv["total_lines_touched"], 0)

        base = {
            "ports": [],
            "totals": {
                "frictions": {"open_blockers": 0, "warnings": 0},
                "tokens": {},
                "friction_lint": {"wa_trap": 0},
                "revision": {"cases": [{"id": "r1", "ok": True, "lines_touched": 7}]},
            },
        }
        cur_grew = {
            "ports": [],
            "totals": {
                "frictions": {"open_blockers": 0, "warnings": 0},
                "tokens": {},
                "friction_lint": {"wa_trap": 0},
                "revision": {"cases": [{"id": "r1", "ok": True, "lines_touched": 12}]},
            },
        }
        diff = sb.diff_against_baseline(cur_grew, base, mode="fast")
        self.assertTrue(any("revision lines_touched grew" in m for m in diff["regressions"]))

    def test_worst_dream_leaderboard_sorting(self):
        import importlib.util

        spec = importlib.util.spec_from_file_location(
            "ports_page", os.path.join(os.path.dirname(os.path.abspath(__file__)), "ports-page.py")
        )
        pp = importlib.util.module_from_spec(spec)
        spec.loader.exec_module(pp)

        def mk(pid, ratio, gap, dream_tok=100):
            return (
                "2026-01-01",
                {"id": pid, "name": pid, "path": f"p/{pid}.cp", "orig_path": f"p/{pid}.cpp"},
                None,
                None,
                0,
                [],
                {"dream_ratio": ratio, "dream_gap_tokens": gap, "dream_tokens": dream_tok, "cp_tokens": (dream_tok or 0) + (gap or 0)},
                None,
            )

        rows = [
            mk("no_dream", None, None, None),
            mk("small", 1.2, 20),
            mk("tie_a", 2.5, 100),
            mk("tie_b", 2.5, 150),
            mk("worst", 4.1, 310),
        ]
        top = pp.worst_dream_leaderboard(rows, limit=3)
        self.assertEqual([r[1]["id"] for r in top], ["worst", "tie_b", "tie_a"])

    def test_spelling_drift_measure_and_ratchet(self):
        with tempfile.TemporaryDirectory() as tmp:
            os.makedirs(os.path.join(tmp, "ecnerwala"))
            os.makedirs(os.path.join(tmp, "scout_batch_1"))
            with open(os.path.join(tmp, "ecnerwala", "a.cp"), "w", encoding="utf-8") as f:
                f.write(
                    "@!multitest\n"
                    "// sum(xs) in comment must be ignored\n"
                    'let msg = "sort(mut xs) in string"\n'
                    "let g1 = sum(x * x for x in xs)\n"
                    "let g2 = first(i for i in 0..n if a[i] > 0)\n"
                    "let s = sum(xs) + xs.sum()\n"
                    "let p = first(xs, x => x > 0)\n"
                    "sort(mut xs)\n"
                )
            with open(os.path.join(tmp, "ecnerwala", "b.cp"), "w", encoding="utf-8") as f:
                f.write("--- multitest ---\nxs.sort()\n")
            with open(os.path.join(tmp, "scout_batch_1", "old.cp"), "w", encoding="utf-8") as f:
                f.write("@multitest\nlet s = sum(a)\n")

            sd = sb.measure_spelling_drift(tmp)
            self.assertEqual(sd["files_scanned"], 3)
            self.assertEqual(sd["active_files_scanned"], 2)
            self.assertEqual(sd["constructs"]["multitest"]["non_canonical"], 2)
            self.assertEqual(sd["constructs"]["multitest"]["active_non_canonical"], 1)
            self.assertEqual(sd["constructs"]["sum"]["non_canonical"], 2)
            self.assertEqual(sd["constructs"]["sum"]["active_non_canonical"], 1)
            self.assertEqual(sd["constructs"]["first"]["non_canonical"], 1)
            self.assertEqual(sd["constructs"]["sort"]["non_canonical"], 1)
            self.assertEqual(sd["total_non_canonical"], 6)
            self.assertEqual(sd["active_non_canonical"], 4)

            base = {
                "ports": [],
                "totals": {
                    "frictions": {"open_blockers": 0, "warnings": 0},
                    "tokens": {},
                    "friction_lint": {"wa_trap": 0},
                    "spelling_drift": sd,
                },
            }
            cur_grew = {
                "ports": [],
                "totals": {
                    "frictions": {"open_blockers": 0, "warnings": 0},
                    "tokens": {},
                    "friction_lint": {"wa_trap": 0},
                    "spelling_drift": {
                        **sd,
                        "total_non_canonical": 7,
                        "constructs": {
                            **sd["constructs"],
                            "sum": {**sd["constructs"]["sum"], "non_canonical": 3},
                        },
                    },
                },
            }
            diff_bad = sb.diff_against_baseline(cur_grew, base, mode="fast")
            self.assertTrue(any("spelling drift" in m for m in diff_bad["regressions"]))
            self.assertTrue(
                any("active/dream spelling drift" in m and "must be 0" in m for m in diff_bad["regressions"])
            )

            cur_better = {
                "ports": [],
                "totals": {
                    "frictions": {"open_blockers": 0, "warnings": 0},
                    "tokens": {},
                    "friction_lint": {"wa_trap": 0},
                    "spelling_drift": {
                        **sd,
                        "total_non_canonical": 5,
                        "active_non_canonical": 0,
                        "constructs": {
                            k: {**v, "active_non_canonical": 0, **({"non_canonical": 1} if k == "sum" else {})}
                            for k, v in sd["constructs"].items()
                        },
                    },
                },
            }
            diff_good = sb.diff_against_baseline(cur_better, base, mode="fast")
            self.assertEqual(diff_good["regressions"], [])
            self.assertTrue(any("spelling drift" in m for m in diff_good["improvements"]))

    def test_fix_spelling_drift_codemod(self):
        import importlib.util

        spec = importlib.util.spec_from_file_location(
            "fix_spelling_drift", os.path.join(sb.ROOT, "scripts", "fix-spelling-drift.py")
        )
        mod = importlib.util.module_from_spec(spec)
        spec.loader.exec_module(mod)
        with tempfile.TemporaryDirectory() as tmp:
            os.makedirs(os.path.join(tmp, "ecnerwala", "dream"))
            os.makedirs(os.path.join(tmp, "scout_batch_1"))
            p1 = os.path.join(tmp, "ecnerwala", "a.cp")
            p2 = os.path.join(tmp, "ecnerwala", "dream", "b.cp")
            p3 = os.path.join(tmp, "ecnerwala", "dream", "c.cp")
            p_old = os.path.join(tmp, "scout_batch_1", "old.cp")
            for path, text in (
                (p1, "@multitest\nlet n = read()\n"),
                (p2, "--- multitest ---\nlet n = read()\n"),
                (p3, "--- with multitest() ---\nlet n = read()\n"),
                (p_old, "@multitest\nlet n = read()\n"),
            ):
                with open(path, "w", encoding="utf-8") as f:
                    f.write(text)
            changed = mod.run_codemod(tmp, write=True)
            self.assertEqual(len(changed), 3)
            self.assertEqual(sb._read_text(p1), "@!multitest\nlet n = read()\n")
            self.assertEqual(sb._read_text(p2), "@!multitest\nlet n = read()\n")
            self.assertEqual(sb._read_text(p3), "@!multitest\nlet n = read()\n")
            self.assertEqual(sb._read_text(p_old), "@multitest\nlet n = read()\n")
            self.assertEqual(sb.measure_spelling_drift(tmp)["active_non_canonical"], 0)

    def test_dream_scores_and_gate_fast_ratchets(self):
        with tempfile.TemporaryDirectory() as tmp:
            for name, score in (("a.md", 3), ("b.md", 4), ("c.md", 2)):
                with open(os.path.join(tmp, name), "w", encoding="utf-8") as f:
                    f.write(f"Score: 4/5\nDream: {score}/5\n")
            ds = sb.summarize_dream_scores(tmp)
            self.assertEqual(ds, {"scored": 3, "median": 3, "under_4": 2, "min": 2, "max": 4})

            gt_path = os.path.join(tmp, "gate-fast-time.json")
            with open(gt_path, "w", encoding="utf-8") as f:
                f.write('{"mode":"fast","elapsed_s":142,"budget_s":180}\n')
            gf = sb.load_gate_fast_time(path=gt_path)
            self.assertEqual(gf, {"elapsed_s": 142.0, "budget_s": 180.0, "ok": True})

            base = {
                "ports": [],
                "totals": {
                    "frictions": {"open_blockers": 0, "warnings": 0},
                    "tokens": {},
                    "friction_lint": {"wa_trap": 0},
                    "dream_scores": {"scored": 3, "median": 3, "under_4": 2, "min": 2, "max": 4},
                    "gate_fast": {"elapsed_s": 142.0, "budget_s": 180.0, "ok": True},
                },
            }
            cur_bad = {
                "ports": [],
                "totals": {
                    "frictions": {"open_blockers": 0, "warnings": 0},
                    "tokens": {},
                    "friction_lint": {"wa_trap": 0},
                    "dream_scores": {"scored": 4, "median": 2.5, "under_4": 3, "min": 2, "max": 4},
                    "gate_fast": {"elapsed_s": 195.0, "budget_s": 180.0, "ok": False},
                },
            }
            diff_bad = sb.diff_against_baseline(cur_bad, base, mode="fast")
            self.assertTrue(any("dream-score median regressed" in m for m in diff_bad["regressions"]))
            self.assertTrue(any("dream-score under-4 count grew" in m for m in diff_bad["regressions"]))
            self.assertFalse(any("check.sh --fast wall time" in m for m in diff_bad["regressions"]))
            self.assertTrue(any("check.sh --fast wall time" in m for m in diff_bad["warnings"]))

            cur_good = {
                "ports": [],
                "totals": {
                    "frictions": {"open_blockers": 0, "warnings": 0},
                    "tokens": {},
                    "friction_lint": {"wa_trap": 0},
                    "dream_scores": {"scored": 4, "median": 4, "under_4": 1, "min": 3, "max": 5},
                    "gate_fast": {"elapsed_s": 120.0, "budget_s": 180.0, "ok": True},
                },
            }
            diff_good = sb.diff_against_baseline(cur_good, base, mode="fast")
            self.assertEqual(diff_good["regressions"], [])
            self.assertTrue(any("dream-score median improved" in m for m in diff_good["improvements"]))
            self.assertTrue(any("dream-score under-4 count shrank" in m for m in diff_good["improvements"]))

    def test_per_port_baseline_merge(self):
        # Simulates the cf2124f2 race: main already updated cf2124f2 to 16334 B,
        # while another branch from an older checkout (where cf2124f2 was 15302 B)
        # only touched cf2084h. Per-port merge must keep cf2124f2 at 16334 B and
        # update only cf2084h.
        def mk_port(pid, eb, sha):
            return {
                "id": pid,
                "path": f"proto/ports/{pid}.cp",
                "orig_path": f"proto/ports/{pid}_orig.cpp",
                "gate": "pass",
                "gate_mode": "fast",
                "gate_detail": "ok",
                "check_ok": True,
                "fmt_ok": True,
                "lint_warnings": 0,
                "taste_lints": 0,
                "taste_lints_by_lint": {},
                "build_ok": True,
                "emitted_bytes": eb,
                "over_48kb": False,
                "over_64kb": False,
                "cp_tokens": 100,
                "orig_tokens": 250,
                "orig_stripped_tokens": 200,
                "token_ratio": 0.4,
                "stripped_token_ratio": 0.5,
                "cp_sha": sha,
                "orig_sha": "orig-sha",
                "has_gen": True,
                "has_samples": True,
                "interactive": False,
                "has_checker": False,
                "perf": {"time_ratio": 1.0, "cpp_ms": 20.0, "orig_ms": 20.0},
                "interp": None,
            }

        existing = {
            "schema_version": 1,
            "meta": {"commit": "main-new", "mode": "fast"},
            "totals": {"gate_fast": {"elapsed_s": 120.0, "budget_s": 180.0, "ok": True}},
            "ports": [
                mk_port("ecnerwala/cf2084h", 9000, "sha-h-old"),
                mk_port("ecnerwala/cf2124f2", 16334, "sha-f2-new"),
            ],
        }
        stale_branch_report = {
            "schema_version": 1,
            "meta": {"commit": "branch-tip", "mode": "fast"},
            "totals": {},
            "ports": [
                mk_port("ecnerwala/cf2084h", 8500, "sha-h-new"),
                mk_port("ecnerwala/cf2124f2", 15302, "sha-f2-old"),
            ],
        }
        stale_branch_report["ports"][0]["perf"] = None
        stale_branch_report["ports"][1]["perf"] = None

        merged = sb.merge_baseline_ports(existing, stale_branch_report, touched_ids={"ecnerwala/cf2084h"})
        by_id = {p["id"]: p for p in merged["ports"]}
        self.assertEqual(by_id["ecnerwala/cf2124f2"]["emitted_bytes"], 16334)
        self.assertEqual(by_id["ecnerwala/cf2124f2"]["cp_sha"], "sha-f2-new")
        self.assertEqual(by_id["ecnerwala/cf2084h"]["emitted_bytes"], 8500)
        self.assertEqual(by_id["ecnerwala/cf2084h"]["cp_sha"], "sha-h-new")
        self.assertEqual(merged["totals"]["emitted_bytes"]["max"], 16334)
        self.assertEqual(merged["totals"]["gate_fast"]["elapsed_s"], 120.0)

        # Path -> port ID mapping handles ports, dreams, gens, checkers, samples, and skips non-set problems.
        self.assertEqual(sb._port_id_from_relpath("corpus/problems/cf2124f2/port.cp"), "ecnerwala/cf2124f2")
        self.assertEqual(sb._port_id_from_relpath("corpus/problems/cf2124f2/dream.cp"), "ecnerwala/cf2124f2")
        self.assertEqual(sb._port_id_from_relpath("corpus/problems/cf2124f2/tests/1.in"), "ecnerwala/cf2124f2")
        self.assertIsNone(sb._port_id_from_relpath("corpus/problems/not_in_set_xyz/port.cp"))

    def test_interp_ratio_section_and_max_p50_ratchet(self):
        base = {
            "ports": [],
            "totals": {
                "frictions": {"open_blockers": 0, "warnings": 0},
                "tokens": {},
                "friction_lint": {"wa_trap": 0},
                "perf": {},
                "interp": {
                    "measured": 10,
                    "passed": 10,
                    "failed": 0,
                    "median_ms": 15.0,
                    "p95_ms": 40.0,
                    "max_ms": 60.0,
                    "max_port": "ecnerwala/a",
                    "ratio_above_floor": 10,
                    "ratio_p50": 6.0,
                    "ratio_p95": 25.0,
                    "ratio_max": 50.0,
                    "ratio_max_port": "ecnerwala/a",
                    "bar_ok": True,
                    "north_star_ok": False,
                    "over_10x": 3,
                },
            },
        }
        cur_bad = {
            "ports": [],
            "totals": {
                "frictions": {"open_blockers": 0, "warnings": 0},
                "tokens": {},
                "friction_lint": {"wa_trap": 0},
                "perf": {},
                "interp": {
                    **base["totals"]["interp"],
                    "ratio_p50": 7.0,
                    "ratio_max": 60.0,
                },
            },
        }
        diff_bad = sb.diff_against_baseline(cur_bad, base, mode="full")
        self.assertTrue(any("interp ratio p50 regressed" in m for m in diff_bad["regressions"]))
        self.assertTrue(any("interp ratio max regressed" in m for m in diff_bad["regressions"]))

        cur_good = {
            "ports": [],
            "totals": {
                "frictions": {"open_blockers": 0, "warnings": 0},
                "tokens": {},
                "friction_lint": {"wa_trap": 0},
                "perf": {},
                "interp": {
                    **base["totals"]["interp"],
                    "ratio_p50": 4.5,
                    "ratio_max": 40.0,
                },
            },
        }
        diff_good = sb.diff_against_baseline(cur_good, base, mode="full")
        self.assertEqual(diff_good["regressions"], [])
        self.assertTrue(any("interp ratio p50 improved" in m for m in diff_good["improvements"]))
        self.assertTrue(any("interp ratio max improved" in m for m in diff_good["improvements"]))

        # Verify print_summary emits the interp-ratio section with p50, p95, max, and >10x.
        import io
        from contextlib import redirect_stdout

        rep = {
            "meta": {"mode": "fast", "commit": "abc1234", "elapsed_s": 1.0},
            "totals": {
                "ports_passed": 1,
                "ports_total": 1,
                "tokens": {"cp_total": 10, "orig_total": 20, "median_ratio": 0.5, "median_stripped_ratio": 0.5, "over_1_0x_raw": 0, "over_1_0x_stripped": 0},
                "emitted_bytes": {"max": None},
                "perf": {"measured": 0},
                "interp": {**base["totals"]["interp"], "cached": True},
                "friction_lint": {"wa_trap": 0, "real_gap": 0, "stale_port": 0},
            },
            "frictions": {"files": 0, "open": 0, "closed": 0, "by_severity": {"blocker": 0, "high": 0, "medium": 0, "low": 0}, "by_goal": {}, "open_blockers": [], "open_highs": [], "warnings": []},
            "ports": [],
        }
        buf = io.StringIO()
        with redirect_stdout(buf):
            sb.print_summary(rep, {"has_baseline": False, "added_ports": [], "removed_ports": [], "regressions": [], "improvements": []}, quiet=True)
        out = buf.getvalue()
        self.assertIn("interp-ratio: p50=6.0x", out)
        self.assertIn("p95=25.0x", out)
        self.assertIn("max=50.0x", out)
        self.assertIn(">10x: 3", out)
        self.assertIn("(cached)", out)

    def test_dream_compile_metric_and_ratchet(self):
        dc = sb.summarize_dream_compile({"dreams": 60, "with_errors": 54})
        self.assertEqual(dc, {"compiled": 6, "total": 60, "rate_pct": 10.0})

        base = {
            "ports": [],
            "totals": {
                "frictions": {"open_blockers": 0, "warnings": 0},
                "tokens": {},
                "friction_lint": {"wa_trap": 0},
                "dream_compile": {"compiled": 6, "total": 60, "rate_pct": 10.0},
            },
        }
        # Breaking a compiling dream must fail.
        cur_bad = {
            "ports": [],
            "totals": {
                **base["totals"],
                "dream_compile": {"compiled": 5, "total": 60, "rate_pct": 8.3},
            },
        }
        diff_bad = sb.diff_against_baseline(cur_bad, base, mode="fast")
        self.assertTrue(any("dream-compile count regressed" in m for m in diff_bad["regressions"]))

        # Adding a new uncompilable dream without breaking existing compiling dreams must pass.
        cur_new_dream = {
            "ports": [],
            "totals": {
                **base["totals"],
                "dream_compile": {"compiled": 6, "total": 62, "rate_pct": 9.7},
            },
        }
        diff_new = sb.diff_against_baseline(cur_new_dream, base, mode="fast")
        self.assertEqual(diff_new["regressions"], [])

        # Compiling more dreams records an improvement.
        cur_good = {
            "ports": [],
            "totals": {
                **base["totals"],
                "dream_compile": {"compiled": 9, "total": 60, "rate_pct": 15.0},
            },
        }
        diff_good = sb.diff_against_baseline(cur_good, base, mode="fast")
        self.assertEqual(diff_good["regressions"], [])
        self.assertTrue(any("dream-compile improved" in m for m in diff_good["improvements"]))

    def test_spelling_drift_construct_split(self):
        with tempfile.TemporaryDirectory() as tmp:
            ec = os.path.join(tmp, "ecnerwala")
            os.makedirs(ec, exist_ok=True)
            with open(os.path.join(ec, "a.cp"), "w", encoding="utf-8") as f:
                f.write(
                    "@multitest\n"
                    "for i in Pos where a[i] > 0:\n"
                    "    let x = a.count(where: v => v > 0)\n"
                    "    let y = two.pow(n) + two ** n\n"
                    "    let z = b.rev() + a.len() + a.len + a.max()\n"
                )
            sd = sb.measure_spelling_drift(tmp)
            cs = sd["constructs"]
            self.assertEqual(cs["multitest"]["active_non_canonical"], 1)
            self.assertEqual(cs["where_filter"]["active_non_canonical"], 2)
            self.assertEqual(cs["dot_pow"]["active_non_canonical"], 1)
            self.assertEqual(cs["dot_pow"]["active_counts"]["a ** b"], 1)
            self.assertEqual(cs["getter_parens"]["active_non_canonical"], 2)
            self.assertEqual(cs["getter_parens"]["active_counts"][".getter"], 1)

            # Growing a newly tracked construct's active drift must fail; reducing it records improvement.
            base = {
                "ports": [],
                "totals": {
                    "frictions": {"open_blockers": 0, "warnings": 0},
                    "tokens": {},
                    "friction_lint": {"wa_trap": 0},
                    "spelling_drift": {
                        "total_non_canonical": 5,
                        "active_non_canonical": 5,
                        "constructs": {
                            "multitest": {"non_canonical": 0, "active_non_canonical": 0},
                            "where_filter": {"non_canonical": 2, "active_non_canonical": 2},
                            "dot_pow": {"non_canonical": 1, "active_non_canonical": 1},
                            "getter_parens": {"non_canonical": 2, "active_non_canonical": 2},
                        },
                    },
                },
            }
            cur_bad = {
                "ports": [],
                "totals": {
                    **base["totals"],
                    "spelling_drift": {
                        "total_non_canonical": 6,
                        "active_non_canonical": 6,
                        "constructs": {
                            "multitest": {"non_canonical": 0, "active_non_canonical": 0},
                            "where_filter": {"non_canonical": 3, "active_non_canonical": 3},
                            "dot_pow": {"non_canonical": 1, "active_non_canonical": 1},
                            "getter_parens": {"non_canonical": 2, "active_non_canonical": 2},
                        },
                    },
                },
            }
            d_bad = sb.diff_against_baseline(cur_bad, base, mode="fast")
            self.assertTrue(any("where_filter" in m for m in d_bad["regressions"]))

            cur_good = {
                "ports": [],
                "totals": {
                    **base["totals"],
                    "spelling_drift": {
                        "total_non_canonical": 3,
                        "active_non_canonical": 3,
                        "constructs": {
                            "multitest": {"non_canonical": 0, "active_non_canonical": 0},
                            "where_filter": {"non_canonical": 1, "active_non_canonical": 1},
                            "dot_pow": {"non_canonical": 0, "active_non_canonical": 0},
                            "getter_parens": {"non_canonical": 2, "active_non_canonical": 2},
                        },
                    },
                },
            }
            d_good = sb.diff_against_baseline(cur_good, base, mode="fast")
            self.assertEqual(d_good["regressions"], [])
            self.assertTrue(any("dot_pow" in m for m in d_good["improvements"]))

    def test_attributable_gate_ratchets(self):
        base = {
            "ports": [
                {"id": "ecnerwala/cf2124f2", "gate": "pass", "emitted_bytes": 15000},
                {"id": "ecnerwala/cf2084h", "gate": "pass", "emitted_bytes": 9000},
            ],
            "totals": {
                "frictions": {"open_blockers": 0, "warnings": 0},
                "tokens": {},
                "friction_lint": {"wa_trap": 0},
            },
        }
        cur = {
            "ports": [
                {"id": "ecnerwala/cf2124f2", "gate": "pass", "emitted_bytes": 17000},
                {"id": "ecnerwala/cf2084h", "gate": "pass", "emitted_bytes": 9000},
            ],
            "totals": {
                "frictions": {"open_blockers": 0, "warnings": 2},
                "tokens": {},
                "friction_lint": {"wa_trap": 0},
            },
        }
        # Commit touching only cf2084h is NOT blocked by untouched cf2124f2 or unrelated friction warnings.
        scope_other_port = {
            "touched_ports": {"ecnerwala/cf2084h"},
            "touched_codegen_or_stdlib": False,
            "codegen_causes": [],
            "touched_frictions": False,
            "touched_reviews": False,
            "touched_dreams": False,
            "touched_goal_tests": False,
        }
        d_other = sb.diff_against_baseline(cur, base, mode="fast", scope=scope_other_port)
        self.assertEqual(d_other["regressions"], [])
        self.assertTrue(any("cf2124f2" in w and "unrelated" in w for w in d_other["warnings"]))

        # Commit touching cf2124f2 IS blocked and names `touched port`.
        scope_same_port = {**scope_other_port, "touched_ports": {"ecnerwala/cf2124f2"}}
        d_same = sb.diff_against_baseline(cur, base, mode="fast", scope=scope_same_port)
        self.assertEqual(len(d_same["regressions"]), 1)
        self.assertIn("ecnerwala/cf2124f2", d_same["regressions"][0])
        self.assertIn("cause: touched port", d_same["regressions"][0])

        # Growth in a touched port whose review is touched too is a reviewed
        # rewrite: it still blocks until the port is rebaselined in the same
        # commit, and the hint names the command that accepts it.
        grow_base = {"ports": [{"id": "ecnerwala/cf9999a", "gate": "pass", "emitted_bytes": 14000}], "totals": base["totals"]}
        grow_cur = {"ports": [{"id": "ecnerwala/cf9999a", "gate": "pass", "emitted_bytes": 22000}], "totals": base["totals"]}
        scope_rewrite = {**scope_other_port, "touched_ports": {"ecnerwala/cf9999a"}, "reviewed_ports": {"cf9999a"}, "touched_reviews": True}
        d_rw = sb.diff_against_baseline(grow_cur, grow_base, mode="fast", scope=scope_rewrite)
        self.assertEqual(len(d_rw["regressions"]), 1)
        self.assertIn("--update ecnerwala/cf9999a", d_rw["regressions"][0])
        d_rebased = sb.diff_against_baseline(grow_cur, grow_cur, mode="fast", scope=scope_rewrite)
        self.assertEqual(d_rebased["regressions"], [])
        d_norev = sb.diff_against_baseline(grow_cur, grow_base, mode="fast", scope={**scope_rewrite, "reviewed_ports": set()})
        self.assertEqual(len(d_norev["regressions"]), 1)
        big_cur = {"ports": [{"id": "ecnerwala/cf9999a", "gate": "pass", "emitted_bytes": 50000}], "totals": base["totals"]}
        d_big = sb.diff_against_baseline(big_cur, grow_base, mode="fast", scope=scope_rewrite)
        self.assertEqual(len(d_big["regressions"]), 1)

        # Commit touching codegen/stdlib IS blocked and names the port + codegen cause.
        scope_codegen = {
            **scope_other_port,
            "touched_ports": set(),
            "touched_codegen_or_stdlib": True,
            "codegen_causes": ["crates/cpl-codegen/src/emit.rs"],
        }
        d_cg = sb.diff_against_baseline(cur, base, mode="fast", scope=scope_codegen)
        self.assertEqual(len(d_cg["regressions"]), 1)
        self.assertIn("ecnerwala/cf2124f2", d_cg["regressions"][0])
        self.assertIn("stdlib/codegen diff (crates/cpl-codegen/src/emit.rs)", d_cg["regressions"][0])

    def test_rewriting_the_checked_in_baseline_is_byte_identical(self):
        loaded = sb.load_baseline(sb.DEFAULT_BASELINE)
        with tempfile.TemporaryDirectory() as tmp:
            out_path = os.path.join(tmp, "scoreboard.json")
            sb.write_baseline(loaded, out_path=out_path, ports_only=True)
            with open(sb.ports_jsonl_path(out_path), encoding="utf-8") as f:
                written = f.read()
        with open(sb.ports_jsonl_path(sb.DEFAULT_BASELINE), encoding="utf-8") as f:
            self.assertEqual(written, f.read())

    def test_merge_safe_jsonl_and_update_cli(self):
        with tempfile.TemporaryDirectory() as tmp:
            out_path = os.path.join(tmp, "scoreboard.json")
            ports = [
                {"id": f"ecnerwala/p{i:02d}", "gate": "pass", "gate_mode": "full", "cp_sha": f"s{i}", "orig_sha": f"o{i}", "emitted_bytes": 1000 + i}
                for i in range(10)
            ]
            rep = {"schema_version": 1, "meta": {"commit": "base"}, "totals": {"ports_total": 10}, "ports": ports}
            sb.write_baseline(rep, out_path=out_path)

            jpath = sb.ports_jsonl_path(out_path)
            self.assertTrue(os.path.isfile(jpath))
            with open(jpath, encoding="utf-8") as f:
                base_lines = f.read()
            self.assertEqual(len(base_lines.splitlines()), 10)
            with open(out_path, encoding="utf-8") as f:
                summary_before = f.read()
            self.assertNotIn("emitted_bytes", summary_before)

            # Round-trip load_baseline reconstructs all 10 sorted ports.
            loaded = sb.load_baseline(out_path)
            self.assertEqual(len(loaded["ports"]), 10)
            self.assertEqual(loaded["ports"][0]["id"], "ecnerwala/p00")

            # update_baseline_ports touches ONLY the named entry in .ports.jsonl, leaving summary JSON untouched.
            sb.update_baseline_ports(
                out_path,
                [{"id": "ecnerwala/p01", "gate": "pass", "gate_mode": "fast", "cp_sha": "s1-new", "orig_sha": "o1", "emitted_bytes": 2222}],
            )
            with open(out_path, encoding="utf-8") as f:
                self.assertEqual(f.read(), summary_before)
            with open(jpath, encoding="utf-8") as f:
                branch_a = f.read()

            # Reset to base and update a distant port on branch B, then prove git merge-file merges cleanly.
            with open(jpath, "w", encoding="utf-8") as f:
                f.write(base_lines)
            sb.update_baseline_ports(
                out_path,
                [{"id": "ecnerwala/p08", "gate": "pass", "gate_mode": "fast", "cp_sha": "s8-new", "orig_sha": "o8", "emitted_bytes": 8888}],
            )
            with open(jpath, encoding="utf-8") as f:
                branch_b = f.read()

            fa, fb, fbase = os.path.join(tmp, "a"), os.path.join(tmp, "b"), os.path.join(tmp, "base")
            for p, content in ((fa, branch_a), (fb, branch_b), (fbase, base_lines)):
                with open(p, "w", encoding="utf-8") as f:
                    f.write(content)
            r = subprocess.run(["git", "merge-file", "-p", fa, fbase, fb], capture_output=True, text=True)
            self.assertEqual(r.returncode, 0, r.stderr)
            merged_by_id = {json.loads(line)["id"]: json.loads(line) for line in r.stdout.splitlines() if line.strip()}
            self.assertEqual(merged_by_id["ecnerwala/p01"]["emitted_bytes"], 2222)
            self.assertEqual(merged_by_id["ecnerwala/p08"]["emitted_bytes"], 8888)
            self.assertEqual(merged_by_id["ecnerwala/p05"]["emitted_bytes"], 1005)

    def test_interp_over10x_and_turbofish_drift_ratchets(self):
        # 1. Interpreter ratio p50 (never worse) and >10x count (never higher)
        base = {
            "ports": [],
            "totals": {
                "frictions": {"open_blockers": 0, "warnings": 0},
                "tokens": {},
                "perf": {},
                "interp": {"ratio_p50": 6.0, "ratio_max": 50.0, "over_10x": 34},
                "friction_lint": {"wa_trap": 0},
            },
        }
        cur_worse = {
            "ports": [],
            "totals": {
                "frictions": {"open_blockers": 0, "warnings": 0},
                "tokens": {},
                "perf": {},
                "interp": {"ratio_p50": 6.1, "ratio_max": 50.0, "over_10x": 35},
                "friction_lint": {"wa_trap": 0},
            },
        }
        d_bad = sb.diff_against_baseline(cur_worse, base, mode="full")
        self.assertTrue(any("interp ratio p50 regressed" in r for r in d_bad["regressions"]))
        self.assertTrue(any("interp ratio >10x count grew: 34 -> 35" in r for r in d_bad["regressions"]))

        cur_better = {
            "ports": [],
            "totals": {
                "frictions": {"open_blockers": 0, "warnings": 0},
                "tokens": {},
                "perf": {},
                "interp": {"ratio_p50": 4.8, "ratio_max": 40.0, "over_10x": 20},
                "friction_lint": {"wa_trap": 0},
            },
        }
        d_good = sb.diff_against_baseline(cur_better, base, mode="full")
        self.assertEqual(d_good["regressions"], [])
        self.assertTrue(any(">10x count shrank: 34 -> 20" in m for m in d_good["improvements"]))

        # 2. Turbofish spelling drift (f::<T>() non-canonical vs f<T>() canonical)
        with tempfile.TemporaryDirectory() as tmp:
            os.makedirs(os.path.join(tmp, "ecnerwala"), exist_ok=True)
            with open(os.path.join(tmp, "ecnerwala", "tf.cp"), "w", encoding="utf-8") as f:
                f.write(
                    "@!multitest\n"
                    "fn solve():\n"
                    "  let a = read<Int>()\n"
                    "  let b = read::<Int>()\n"
                    "  let c = topo::<V>(g)\n"
                )
            sd = sb.measure_spelling_drift(tmp)
            tf = sd["constructs"]["turbofish"]
            self.assertEqual(tf["canonical"], "f<T>(...)")
            self.assertEqual(tf["counts"]["f<T>(...)"], 1)
            self.assertEqual(tf["counts"]["f::<T>(...)"], 2)
            self.assertEqual(tf["active_non_canonical"], 2)

    def test_taste_review_formats_stale_and_unreviewed(self):
        # 1. Parser handles every review format and never confuses pending or Dream: lines.
        cases = [
            ("# a\n\nScore: 4/5\n", (4, False)),
            ("# b\n\n- **Score:** 3 / 5\n", (3, False)),
            ("# c\n\n**Port score:** 5 out of 5\n", (5, False)),
            ("# d\n\nTaste score — 2\n", (2, False)),
            ("# e\n\n## Score\n\n**4/5**\n", (4, False)),
            ("# f\n\nScore: 3/5 (was 2/5)\n", (3, False)),
            ("# g\n\nScore: pending (was 2/5: rewritten)\nDream: 4/5\n", (None, True)),
            ("# h\n\nDream: 5/5\nNo port score yet.\n", (None, False)),
        ]
        for text, expected in cases:
            self.assertEqual(sb.parse_review_score_text(text), expected, msg=text)

        # 2. summarize_taste_reviews includes non-active/rewrite-lane review files,
        # computes taste_stale via git commit order, and counts taste_unreviewed.
        with tempfile.TemporaryDirectory() as tmp:
            # A commit hook exports GIT_DIR and GIT_INDEX_FILE; inherited, they aim these at the real repo.
            env = {k: v for k, v in os.environ.items() if not k.startswith("GIT_")}

            def git(*args):
                subprocess.run(["git", *args], cwd=tmp, env=env, check=True, capture_output=True, text=True)

            git("init", "-q")
            git("config", "user.email", "test@example.com")
            git("config", "user.name", "Test")

            probs = os.path.join(tmp, "corpus", "problems")
            rev = os.path.join(tmp, "research", "reviews")
            os.makedirs(rev, exist_ok=True)

            # Commit 1: initial ports + stale port's old review + same-commit port/review + unreviewed port
            for name in ("fresh", "stale", "same", "unrev", "extra"):
                os.makedirs(os.path.join(probs, name), exist_ok=True)
                with open(os.path.join(probs, name, "port.cp"), "w", encoding="utf-8") as f:
                    f.write("print(1)\n")
            with open(os.path.join(rev, "README.md"), "w", encoding="utf-8") as f:
                f.write("# Reviews\nScore: 1/5\n")
            with open(os.path.join(rev, "stale.md"), "w", encoding="utf-8") as f:
                f.write("# stale\n\nScore: 2/5\n")
            with open(os.path.join(rev, "same.md"), "w", encoding="utf-8") as f:
                f.write("# same\n\n**Port score:** 3/5\n")
            with open(os.path.join(rev, "extra.md"), "w", encoding="utf-8") as f:
                f.write("# extra\n\n## Score\n\n5/5\n")
            git("add", ".")
            git("commit", "-q", "-m", "initial")

            # Commit 2: add review for fresh (newer than .cp -> NOT stale) and edit stale/port.cp (newer than review -> STALE)
            with open(os.path.join(rev, "fresh.md"), "w", encoding="utf-8") as f:
                f.write("# fresh\n\n- **Score:** 4/5\n")
            with open(os.path.join(probs, "stale", "port.cp"), "w", encoding="utf-8") as f:
                f.write("print(2)\n")
            with open(os.path.join(rev, "pend.md"), "w", encoding="utf-8") as f:
                f.write("# pend\n\nScore: pending (was 2/5)\n")
            git("add", ".")
            git("commit", "-q", "-m", "update")

            port_records = [
                {
                    "id": f"ecnerwala/{n}",
                    "name": n,
                    "path": f"corpus/problems/{n}/port.cp",
                    "gate": "pass",
                    "cp_tokens": 10,
                    "orig_tokens": 20,
                    "orig_stripped_tokens": 20,
                    "token_ratio": 0.5,
                    "stripped_token_ratio": 0.5,
                    "emitted_bytes": 1000,
                }
                for n in ("fresh", "stale", "same", "unrev")
            ]
            summary = sb.summarize_taste_reviews(port_records, reviews_dir=rev, repo_root=tmp)
            self.assertEqual(summary["reviewed"], 4)  # fresh(4), stale(2), same(3), extra(5); README & pend excluded
            self.assertEqual(summary["pending"], 1)
            self.assertEqual(summary["median"], 3.5)
            self.assertEqual(summary["histogram"], {"1": 0, "2": 1, "3": 1, "4": 1, "5": 1})
            self.assertEqual(summary["stale"], 1)  # only stale.cp changed after its review
            self.assertEqual(summary["unreviewed"], 1)  # only unrev has no review file

            totals = sb.compute_totals(
                port_records,
                {"wa_trap": 0, "real_gap": 0, "stale_port": 0, "total_findings": 0},
                {"files": 0, "total": 0, "open": 0, "closed": 0, "by_severity": {}, "by_goal": {}, "open_blockers": [], "open_highs": []},
                taste_summary=summary,
            )
            self.assertEqual(totals["taste_stale"], 1)
            self.assertEqual(totals["taste_unreviewed"], 1)
            self.assertEqual(totals["taste"]["reviewed"], 4)


if __name__ == "__main__":
    unittest.main()
