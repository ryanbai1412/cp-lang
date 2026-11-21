#!/usr/bin/env python3
"""Each friction_lint rule fires on its workaround and not on the clean form.

    python3 scripts/test_friction_lint.py
"""
import os, sys, unittest

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
import friction_lint as fl

# rule -> (snippet it must flag, snippet it must not flag)
CASES = {
    "negative_slice": ("let h = lower[..-1]\n", "let h = lower[..k - 1]\n"),
    "manual_cursor_loop": ("while head < q.len:\n    head += 1\n", "while head < n:\n    head += 1\n"),
    "second_from_end": ("let p = c[c.len - 2]\n", "let p = c[d.len - 2]\n"),
    "domain_step_unwrap": ("let x = (i + 1)!\n", "let x = (i + 1)\n"),
    "sorted_index_list": ("let o = sorted([i for i in 0..n], by: k)\n", "let o = sorted(xs, by: k)\n"),
    "bool_list_loop": ("for b in [false, true]:\n    f(b)\n", "for b in [x, y]:\n    f(b)\n"),
    "infinite_capacity": ("var g = FlowGraph<V>.new\ng.add(u, v, INF)\n",
                          "var st = SegTree<I, Int>(+, a)\nst.add(l, l, -INF)\n"),
    "param_copy": ("fn f(x: Int) -> Int:\n    var x = x\n    x\n", "fn f(x: Int) -> Int:\n    var y = x\n    y\n"),
    "bit_test": ("if x >> i & 1 == 1: f()\n", "if x >> i == 1: f()\n"),
    "digit_loop": ("while x > 0:\n    s += x % 10\n    x /= 10\n", "while x > 0:\n    x /= 2\n"),
    "list_self_append": ("xs = xs + [1]\n", "xs = ys + [1]\n"),
    "segtree_get_set": ("t.set(i, t.get(i) + 1)\n", "t.set(i, 1)\n"),
    "child_end": ("let c = if tree.parent(u) == v: u else: v\n", "let c = tree.parent(u)\n"),
    "arrow_block": ("let s = [V] v =>\n    f(v)\n", "match x:\n    A =>\n        f()\n"),
    "mod_literal_wrapping": ("type Z = Mod<P>\nlet a = Z(0)\n", "type Z = Mod<P>\nlet a: Z = 0\n"),
    "missing_compound_assign": ("a[i] = a[i] + x\n", "a[i] += x\n"),
    "manual_temp_swap": ("let t = a[i]\na[i] = a[j]\na[j] = t\n", "swap(mut a[i], mut a[j])\n"),
    "comprehension_copy": ("let b = [x for x in a]\n", "let b = [x * 2 for x in a]\n"),
    "domain_bound_boilerplate": ("for v in V.first.l..=V.last:\n    f(v)\n", "for v in V:\n    f(v)\n"),
    "positional_action": ("let a = Action<Int, Int>(0, f)\n", "let a = Action<Int, Int>(id: 0, apply: f)\n"),
    "len_minus_one": ("let x = xs[xs.len - 1]\n", "let x = xs.last!\n"),
    "nonempty_then_last": ("while st.len > 0 and st.last! < x: st.pop()\n", "while st.len > 0: st.pop()\n"),
    "min_max_assign": ("best = min(best, d)\n", "best min= d\n"),
    "prototype_stamp": ("// cp: 0.1\nlet n = read()\n", "// cp lang port\nlet n = read()\n"),
    "prelude_import": ("use std.mod.{Mod}\n", "use std.poly.{Poly}\n"),
    "sentinel_minus_one": ("var p = List.filled(n, -1)\n", "var p = List.filled(n, 0)\n"),
    "none_then_assign": ("if m == none: m = 0\n", "m ??= 0\n"),
    "manual_ceil_div": ("let b = (n + k - 1) / k\n", "let b = ceil_div(n, k)\n"),
    "bool_to_int": ("s += if c: 1 else: 0\n", "s += Int(c)\n"),
    "dead_tail": ("fn f() -> Bool:\n    while true:\n        return g()\n    true\n",
                  "fn f() -> Bool:\n    while g():\n        h()\n    true\n"),
    "push_after_literal": ("var xs = []\nfor i in 0..n: xs.push(i)\n", "var xs = []\nlet y = 1\n"),
    "lone_sum": ("for x in xs: s += x\n", "for x in xs: s += x * x\n"),
    "one_based_shift": ("for i in 1..=n:\n    f(a[i - 1])\n", "for i in 1..=n:\n    f[i] = f[i - 1] + 1\n"),
    "hand_modular": ("fn pow_mod(a: Int, e: Int) -> Int = 1\n", "fn power(a: Int, e: Int) -> Int = 1\n"),
    "manual_kahn": ("var indeg = [0]\nwhile head < q.len:\n    head += 1\n", "var deg = [0]\nlet q = peel(g)\n"),
    "none_check_unwrap": ("if x != none and x! > 3: f()\n", "if let v = x and v > 3: f()\n"),
    "none_or_compare": ("if b == none or d < b!: b = d\n", "b min= d\n"),
    "where_guard": ("let k: 1..=9 = read() where k <= n\n", "let k: 1..=9 = read()\nassert k <= n\n"),
    "mod_list_print": ("print([x.v for x in ans])\n", "print(ans)\n"),
    "doubling_search": ("while ok(hi):\n    hi *= 2\nlet k = bsearch_first(0..hi, ok)\n",
                        "while ok(hi):\n    hi *= 2\n"),
    "bit_len_import": ("use std.math.{bit_len}\n", "use std.math.{bit_width}\n"),
    "hand_decimal_mod": ("x = (x * 10 + (c - '0')) % P\n", "x = x * 10 + (c - '0')\n"),
    "manual_get_or_insert": ("if k not in m: m[k] = 0\n", "if k in m: m[k] = 0\n"),
    "label_crossing": ("let i = v.to_idx\n", "let i = v\n"),
}


def hits(rule, src):
    table = {k: f for k, _, _, _, f in fl.RULES}
    table.update({k: f for k, _, _, f in fl.METRICS})
    code = fl.strip_code(src)
    return table[rule](code, code.split("\n"), src.split("\n"))


class Rules(unittest.TestCase):
    def test_every_rule_has_a_case(self):
        names = {r[0] for r in fl.RULES} | {m[0] for m in fl.METRICS}
        self.assertEqual(names, set(CASES))

    def test_severities(self):
        for key, sev, *_ in fl.RULES:
            self.assertIn(sev, fl.SEVERITIES, key)

    def test_rules(self):
        for rule, (bad, good) in CASES.items():
            with self.subTest(rule=rule):
                self.assertTrue(hits(rule, bad), f"{rule} missed:\n{bad}")
                self.assertFalse(hits(rule, good), f"{rule} flagged:\n{good}")

    def test_flags(self):
        import subprocess
        tool = os.path.join(os.path.dirname(os.path.abspath(__file__)), "friction_lint.py")
        with __import__("tempfile").TemporaryDirectory() as d:
            for name, src in [("clean.cp", "let x = 1\n"), ("trap.cp", "let h = lower[..-1]\n")]:
                with open(os.path.join(d, name), "w") as fh:
                    fh.write(src)
            run = lambda f, *a: subprocess.run([sys.executable, tool, *a, os.path.join(d, f)],
                                               capture_output=True, text=True)
            ok = run("clean.cp", "--summary", "--fail-on-wa-trap")
            self.assertEqual(ok.returncode, 0, ok.stderr)
            self.assertEqual(ok.stdout.count("\n"), 1)
            self.assertIn("0 wa_trap", ok.stdout)
            bad = run("trap.cp", "--summary", "--fail-on-wa-trap")
            self.assertEqual(bad.returncode, 1)
            self.assertIn("negative_slice", bad.stderr)

    def test_comments_and_strings_are_ignored(self):
        self.assertFalse(hits("bool_to_int", '// s += if c: 1 else: 0\nprint("if c: 1 else: 0")\n'))


if __name__ == "__main__":
    unittest.main()
