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


class CliSmokeTest(unittest.TestCase):
    def test_frictions_cli_and_fast_subset_cli(self):
        tool = os.path.join(os.path.dirname(os.path.abspath(__file__)), "scoreboard.py")
        r = subprocess.run([sys.executable, tool, "--frictions", "--json"], capture_output=True, text=True)
        self.assertEqual(r.returncode, 0, r.stderr)
        data = json.loads(r.stdout)
        self.assertIn("by_severity", data)
        self.assertIn("by_goal", data)


if __name__ == "__main__":
    unittest.main()
