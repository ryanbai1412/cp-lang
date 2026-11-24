#!/usr/bin/env python3
"""Unit tests for scripts/std-compose.py: signature parsing and case construction, no compiler."""
import importlib.util
import os
import unittest

SCRIPT = os.path.join(os.path.dirname(os.path.abspath(__file__)), "std-compose.py")
SPEC = importlib.util.spec_from_file_location("std_compose", SCRIPT)
assert SPEC and SPEC.loader
MOD = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(MOD)


def fn(sig, get=False):
    return {**MOD.parse(sig), "get": get, "where": "std/seq.cp:1"}


class StdComposeTest(unittest.TestCase):
    def test_parse_reads_generics_params_and_return(self):
        f = MOD.parse("pub fn windows<S: Indexed>(self: S, k: 1..) -> yields Slice<S>")
        self.assertEqual(f["gens"], {"S": "Indexed"})
        self.assertEqual([(p["name"], p["ty"]) for p in f["params"]], [("self", "S"), ("k", "1..")])
        self.assertEqual(f["ret"], "yields Slice<S>")

    def test_parse_keeps_nested_generics_and_defaults(self):
        f = MOD.parse("pub fn keyed<S: Seq<T>, I: Indexed, T, K>(self: S, keys: I = 0.., by: ?(T -> K) = None)"
                      " -> Keys<S, I, T, K>")
        self.assertEqual(f["gens"]["S"], "Seq<T>")
        self.assertEqual([p["default"] for p in f["params"]], [None, "0..", "None"])

    def test_slot_trait_takes_bare_and_parameterized_bounds(self):
        f = MOD.parse("pub fn zip<A: Seq<T>, B: Indexed, T>(a: A, b: B, mut c: A) -> Int")
        self.assertEqual([MOD.slot_trait(f, p) for p in f["params"]], ["Seq", "Indexed", None])

    def test_fill_synthesizes_sizes_ranges_and_lambdas(self):
        f = fn("pub fn take<S: Indexed>(self: S, k: Len) -> Slice<S>")
        self.assertEqual(MOD.fill(f, {"self": "xs"}, "List(xs)[0]", {}), {"self": "xs", "k": "2"})
        g = fn("pub fn filter<S: Seq<T>, T>(self: S, f: T -> Bool) -> yields T")
        self.assertEqual(MOD.fill(g, {"self": "xs"}, "0", {})["f"], MOD.PRED)
        self.assertIsNone(MOD.fill(fn("pub fn f<S: Seq>(self: S, g: Graph) -> Int"), {"self": "xs"}, "0", {}))

    def test_call_uses_method_syntax_and_getters(self):
        rev = fn("pub fn rev<S: Bidir>(self: S) -> Rev<S>", get=True)
        self.assertEqual(MOD.call(rev, {"self": "xs"}), "(xs).rev")
        zip = fn("pub fn zip<A: Seq, B: Seq>(a: A, b: B) -> Zipped<A, B>")
        self.assertEqual(MOD.call(zip, {"a": "xs", "b": "cs"}), "zip(xs, cs)")

    def test_materialized_keeps_keys(self):
        self.assertEqual(MOD.materialized("v", "Seq"), "List(v)")
        self.assertEqual(MOD.materialized("v", "Indexed"), "List(v).rekey((v).keys)")
        self.assertIn(".keyed(keys:", MOD.materialized("v", "Keyed"))

    def test_every_consumer_slot_and_view_is_a_unit(self):
        rev = fn("pub fn rev<S: Bidir>(self: S) -> Rev<S>", get=True)
        total = fn("pub fn total<S: Seq<T>, T>(self: S) -> T")
        is_view = lambda ret: ret.startswith("Rev")
        units = MOD.cases([rev, total], [rev], [], [], is_view)
        self.assertIn(("view", "total", "self", "rev"), units)
        self.assertIn(("view", "total", "self", None), units)
        lazy = {x["lazy"]: x["brute"] for x in units[("view", "total", "self", "rev")]}
        self.assertEqual(lazy["((@xs).rev).total()"], "(List((@xs).rev)).total()")
        self.assertIn("(@rs).total()", {x["lazy"] for x in units[("view", "total", "self", None)]})
        x = next(x for x in units[("view", "total", "self", "rev")] if x["kind"] == "xs")
        self.assertEqual(MOD.based(x, 4)["lazy"], "((xs4).rev).total()")
        self.assertEqual(MOD.based(x, 4)["base"], "xs4")

    def test_size_reads_the_cost_lines(self):
        perms = {**fn("pub fn permutations<S: Indexed, T>(self: S) -> yields List<T>"),
                 "cost": "\\(O(k)\\) per permutation"}
        total = fn("pub fn total<S: Seq<T>, T>(self: S) -> T")
        self.assertEqual(MOD.size("xs", total), 6)
        self.assertEqual(MOD.size("xs", perms), 6)
        self.assertEqual(MOD.size("xs", total, MOD.math.factorial), 6)
        self.assertEqual(MOD.size("xs", perms, MOD.math.factorial), 3)
        pairs = {**total, "cost": "\\(O(n^2)\\) to walk, \\(O(1)\\) per pair"}
        self.assertEqual(MOD.size("xs", pairs, MOD.math.factorial), 4)
        self.assertEqual(MOD.size("xs", perms, lambda n: 2 * n), 3)

    def test_fill_passes_graphs_and_seq_returning_fns(self):
        reach = fn("pub fn reachable<V, G: Next<V>, S: Seq>(g: G, from: S) -> yields V")
        self.assertEqual(MOD.fill(reach, {"from": "vs3"}, "0", {})["g"], "gr")
        fm = fn("pub fn flat_map<S: Seq<T>, T, U: Seq<V>, V>(self: S, f: T -> U) -> yields V")
        self.assertEqual(MOD.fill(fm, {"self": "xs6"}, "0", {})["f"], "x => [x, x]")

    def test_normal_falls_back_to_show(self):
        sa = fn("pub fn suffix_array<S: Indexed<K, T>, K, T: Ord>(s: S) -> SuffixArray<T>")
        self.assertEqual(MOD.normal(sa, "e", lambda r: False), ["e", '"{e}"'])
        cmp = fn("pub fn compress<S: Seq, T: Ord>(self: S) -> (domain C, Compressed<C, T>)")
        self.assertEqual(MOD.normal(cmp, "e", lambda r: False)[1], '"{(e).1}"')


if __name__ == "__main__":
    unittest.main()
