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

        base = make_rep([("t/fast", 1.0, 50.0), ("t/fast2", 1.0, 50.0), ("t/tiny", 1.0, 2.0), ("t/slow", 2.0, 100.0)])
        self.assertTrue(base["totals"]["perf"]["p50_ok"])
        self.assertFalse(base["totals"]["perf"]["max_ok"])
        self.assertEqual(base["totals"]["perf"]["over_1_5x"], 1)
        self.assertEqual(base["totals"]["perf"]["worst_10"][0]["id"], "t/slow")

        # Sub-floor (<10ms) fluctuation should NOT trigger ratchet; >10% on >=10ms port MUST trigger
        cur_ok = make_rep([("t/fast", 1.05, 50.0), ("t/fast2", 1.05, 50.0), ("t/tiny", 2.5, 2.0), ("t/slow", 2.05, 100.0)])
        diff_ok = sb.diff_against_baseline(cur_ok, base, mode="full")
        self.assertEqual(diff_ok["regressions"], [])

        cur_reg = make_rep([("t/fast", 1.15, 50.0), ("t/fast2", 1.0, 50.0), ("t/tiny", 1.0, 2.0), ("t/slow", 2.0, 100.0)])
        diff_reg = sb.diff_against_baseline(cur_reg, base, mode="full")
        self.assertTrue(any("runtime regressed >10%: t/fast" in m for m in diff_reg["regressions"]))

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
        self.assertTrue(any("interp ratio p50 regressed >10%" in m for m in diff_full["regressions"]))

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
        self.assertGreaterEqual(br["total"], 10)
        self.assertTrue(br["bar_ok"], br)
        self.assertGreaterEqual(br["handled_pct"], 90.0)
        self.assertEqual(br["unexpected_pass"], 0)

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
        self.assertGreaterEqual(rv["total"], 2)
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


if __name__ == "__main__":
    unittest.main()
