//! What an error says and offers, read at 3am.

use crate::tests::{check, messages, one};

fn fixed(src: &str) -> String {
    let diags = check(src);
    let fix = &diags[0].fixes[0];
    let mut out = src.to_string();
    for (span, text) in fix.edits.iter().rev() {
        out.replace_range(span.start as usize..span.end as usize, text);
    }
    out
}

#[test]
fn a_did_you_mean_keeps_values_and_types_apart() {
    use crate::check::closest;
    assert_eq!(closest("cnt", ["Int", "count"].into_iter()), None);
    assert_eq!(closest("vec", ["Vec"].into_iter()).as_deref(), Some("Vec"));
    assert_eq!(
        closest("Lst", ["List"].into_iter()).as_deref(),
        Some("List")
    );
}

#[test]
fn a_number_added_to_a_string_literal_goes_inside_it() {
    assert_eq!(fixed("let x = \"a\" + 1\n"), "let x = \"a{1}\"\n");
    assert_eq!(
        fixed("let n = 2\nprint(n + \"b\")\n"),
        "let n = 2\nprint(\"{n}b\")\n"
    );
    assert_eq!(
        messages(&check("let n = 2\nprint(n + \"b\")\n")),
        ["`+` can't combine an Int and a Str"]
    );
}

#[test]
fn one_place_lent_twice_says_so() {
    let src = "fn f(mut a: Int, mut b: Int):\n    a += b\n    b += 1\nvar x = 0\nf(mut x, mut x)\n";
    let diags = check(src);
    assert_eq!(
        messages(&diags),
        ["`x` is lent to this call twice, both times `mut`: a call can't change one place through two names"]
    );
    assert_eq!(diags[0].help.as_deref(), Some("pass `x` once"));
}

#[test]
fn calling_a_number_suggests_the_product() {
    let diags = check("let x = 5\nprint(x(1))\n");
    assert_eq!(
        diags[0].help.as_deref(),
        Some("to multiply, write `x * (1)`")
    );
}

#[test]
fn an_uppercase_literal_let_becomes_a_const() {
    assert_eq!(fixed("let N = 5\nprint(N)\n"), "const N = 5\nprint(N)\n");
    let read = fixed("let N = read()\nprint(N)\n");
    assert_eq!(read, "let n = read()\nprint(n)\n");
    let var = fixed("var N = 5\nN += 1\nprint(N)\n");
    assert_eq!(var, "var n = 5\nn += 1\nprint(n)\n");
}

#[test]
fn a_misspelled_stdlib_module_names_the_closest() {
    let diags = check("use std.segtre\nprint(1)\n");
    assert_eq!(
        diags[0].help.as_deref(),
        Some("did you mean `std.segtree`?")
    );
    assert_eq!(
        fixed("use std.segtre\nprint(1)\n"),
        "use std.segtree\nprint(1)\n"
    );
    let diags = check("use std.no_such_module\nprint(1)\n");
    assert!(diags[0].fixes.is_empty(), "{diags:?}");
}

#[test]
fn another_languages_true_and_null_are_spelled_here() {
    assert_eq!(
        fixed("let b = True\nprint(b)\n"),
        "let b = true\nprint(b)\n"
    );
    let diags = check("let x: ?Int = null\nprint(x ?? 0)\n");
    assert_eq!(diags[0].help.as_deref(), Some("CP Lang spells it `None`"));
}

#[test]
fn another_languages_member_names_point_at_ours() {
    let diags = check("let xs = [3, 1]\nprint(xs.length())\n");
    assert_eq!(
        diags[0].help.as_deref(),
        Some("did you mean `len`?"),
        "{diags:?}"
    );
}

#[test]
fn pythons_range_says_how_a_range_is_written() {
    let diags = check("for i in range(3):\n    print(i)\n");
    assert!(
        diags[0]
            .help
            .as_deref()
            .is_some_and(|h| h.contains("`0..3`")),
        "{diags:?}"
    );
    assert_eq!(diags[0].fixes[0].edits[0].1, "0..3", "{diags:?}");
}

#[test]
fn pythons_conversions_say_how_ours_are_written() {
    let help = |src: &str| check(src)[0].help.clone().unwrap_or_default();
    assert!(help("let x = 5\nprint(str(x))\n").contains("`\"{x}\"`"));
    assert!(help("let s = \"1\"\nprint(int(s))\n").contains("s.parse<Int>()!"));
    assert!(help("let a = 4\nprint(math.sqrt(a))\n").contains("`a.sqrt`"));
    let d = check("let x = 5\nprint(x.toString())\n");
    assert!(
        d[0].message
            .starts_with("an `Int` has no method `toString`"),
        "{d:?}"
    );
    assert!(
        d[0].help.as_deref().unwrap_or("").contains("`\"{x}\"`"),
        "{d:?}"
    );
}

#[test]
fn a_pairs_first_and_second_name_the_tuple_parts() {
    assert_eq!(
        fixed("let p = (1, 2)\nprint(p.first, p.second)\n"),
        "let p = (1, 2)\nprint(p.0, p.second)\n"
    );
    let d = check("let p = (1, 2)\nprint(p.first, p.second)\n");
    assert_eq!(d.len(), 2, "{d:?}");
    assert!(d[0].message.starts_with("a tuple has no `.first`"), "{d:?}");
    assert_eq!(d[1].fixes[0].edits[0].1, "1", "{d:?}");
}

#[test]
fn empty_braces_for_a_container_offer_its_empty_spelling() {
    let fix = |src: &str| fixed(src).lines().next().unwrap_or("").to_string();
    assert_eq!(
        fix("var m: Map<Int, Int> = {}\nm[1] = 2\n"),
        "var m: Map<Int, Int> = [:]"
    );
    assert_eq!(
        fix("var s: Set<Int> = {}\ns.insert(1)\n"),
        "var s: Set<Int> = Set()"
    );
    let d = check("var m: Map<Int, Int> = {}\n");
    assert!(d[0].message.contains("empty block"), "{d:?}");
}

#[test]
fn in_on_a_list_offers_the_scan() {
    let src = "let xs = [1, 2, 3]\nprint(yes(2 in xs), yes(4 not in xs))\n";
    let d = check(src);
    let scans: Vec<_> = d.iter().map(|d| d.fixes[0].edits[0].1.as_str()).collect();
    assert_eq!(scans, ["xs.contains(2)", "not xs.contains(4)"], "{d:?}");
}

#[test]
fn enumerate_and_reversed_become_members() {
    let src = "let xs = [1, 2]\nfor (i, x) in enumerate(xs):\n    print(i, x)\nfor x in reversed(xs):\n    print(x)\nprint(pow(2, xs.len + 1))\nprint(\"ab\".charAt(1))\n";
    let d = check(src);
    let fixes: Vec<_> = d.iter().map(|d| d.fixes[0].edits[0].1.as_str()).collect();
    assert_eq!(
        fixes,
        ["xs.items", "xs.rev", "2 ** (xs.len + 1)", "\"ab\"[1]"],
        "{d:?}"
    );
}

#[test]
fn pow_with_an_op_is_std_fast_power() {
    let d = check("let xs = [1, 2]\nprint(pow(3, xs.len + 1, op: *), pow(3, 4, op: +))\n");
    assert!(d.is_empty(), "{d:?}");
}

#[test]
fn pow_of_a_modulus_is_pow_mod() {
    let d = one("print(pow(2, 10, 1_000_000_007))\n");
    assert_eq!(
        d.fixes[0].edits[0].1, "pow_mod(2, 10, 1_000_000_007)",
        "{d:?}"
    );
}

#[test]
fn a_modulus_is_part_of_the_mod_type() {
    let d = one("let a: Mod<7> = 3\nlet b: Mod<11> = a\n");
    assert_eq!(
        (d.code.0, d.message.as_str()),
        ("E0302", "`b` is declared a Mod<11>, but `a` is a Mod<7>")
    );
    let d = one("let a: Mod<7> = 3\nlet b: Mod<11> = 4\nprint(a + b)\n");
    assert_eq!(
        (d.code.0, d.message.as_str()),
        ("E0302", "`+` mixes a Mod<7> and a Mod<11>")
    );
}

#[test]
fn max_of_one_sequence_becomes_the_member() {
    let d = check("let xs = [1, 2]\nprint(max(xs), min(xs.rev))\n");
    let fixes: Vec<_> = d.iter().map(|d| d.fixes[0].edits[0].1.as_str()).collect();
    assert_eq!(fixes, ["xs.max()!", "xs.rev.min()!"], "{d:?}");
}

#[test]
fn foreign_case_functions_become_to_upper_and_to_lower() {
    let d = check("let s = \"ab\"\nprint(s.toUpperCase(), s.lowercase())\n");
    let fixes: Vec<_> = d.iter().map(|d| d.fixes[0].edits[0].1.as_str()).collect();
    assert_eq!(fixes, ["to_upper", "to_lower"], "{d:?}");
}

#[test]
fn reversing_a_let_offers_the_view() {
    let d = check("let s = \"abc\"\ns.reverse()\n");
    assert_eq!(d.len(), 1, "{d:?}");
    assert!(d[0].help.as_deref().unwrap().ends_with("`s.rev`"), "{d:?}");
    assert!(d[0].fixes.iter().any(|f| f.edits[0].1 == "var"), "{d:?}");
}

#[test]
fn a_written_show_is_the_print_hook_whatever_the_fields() {
    let src = "struct B(neg: Bool, v: Int)\nimpl Show for B:\n    fn show(a) = if a.neg: \"-{a.v}\" else: \"{a.v}\"\nprint(B(neg: true, v: 3))\n";
    let d = check(src);
    assert!(d.is_empty(), "{:?}", messages(&d));
}

#[test]
fn an_arrays_range_elements_do_not_widen_on_a_move() {
    let d = one("let xs: [3]1..=9 = [1, 2, 3]\nvar d: [3]Int = xs\nprint(d)\n");
    assert_eq!(
        d.message,
        "`d` is declared a [3]Int, but `xs` is a [3]1..=9"
    );
    assert_eq!(
        d.help.as_deref(),
        Some("convert each cell: `xs.map(x => Int(x))`")
    );
    assert!(check("var a: [3]0..=9 = [3].filled(0)\nprint(a)\n").is_empty());
}

#[test]
fn a_number_getter_called_with_parentheses_drops_them() {
    let d = check("let x = -3\nlet f = 2.5\nprint(x.abs(), f.floor())\n");
    let codes: Vec<_> = d
        .iter()
        .map(|d| (d.code.0, d.fixes[0].edits[0].1.as_str()))
        .collect();
    assert_eq!(codes, [("E0304", ""), ("E0304", "")], "{d:?}");
}

#[test]
fn substr_and_substring_become_slices() {
    let d = check("let s = \"abcd\"\nprint(s.substr(1, 2), s.substring(1, 3))\n");
    let fixes: Vec<_> = d.iter().map(|d| d.fixes[0].edits[0].1.as_str()).collect();
    assert_eq!(fixes, ["s[1..1 + 2]", "s[1..3]"], "{d:?}");
}

#[test]
fn int_of_input_is_one_error_with_the_read_fix() {
    let d = check("let n = int(input())\nlet x = float(input())\nprint(n, x)\n");
    let fixes: Vec<_> = d.iter().map(|d| d.fixes[0].edits[0].1.as_str()).collect();
    assert_eq!(fixes, ["read<Int>()", "read<F64>()"], "{d:?}");
    let d = check("let s = input()\nprint(s)\n");
    assert_eq!(d.len(), 1, "{d:?}");
    assert!(d[0].help.as_deref().unwrap().contains("read()"), "{d:?}");
}

#[test]
fn a_copying_twin_of_an_in_place_method_names_the_in_place_one() {
    let d = check("let xs = [1, 2]\nprint(xs.shuffled())\n");
    assert_eq!(d.len(), 1, "{d:?}");
    let help = d[0].help.as_deref().unwrap_or_default();
    assert!(
        help.contains("`var ys = xs.clone()`, then `rng.shuffle(mut ys)`"),
        "{d:?}"
    );
}

#[test]
fn foreign_strip_names_become_trim() {
    let d = check("let s = \" a \"\nprint(s.strip(), s.lstrip(), s.trimEnd())\n");
    let fixes: Vec<_> = d.iter().map(|d| d.fixes[0].edits[0].1.as_str()).collect();
    assert_eq!(fixes, ["trim", "trim_start", "trim_end"], "{d:?}");
}

#[test]
fn an_optional_print_says_how_to_give_none_a_text() {
    let d = check("let x: ?Int = None\nprint(x + 1)\n");
    assert_eq!(d.len(), 1, "{d:?}");
    let help = d[0].help.as_deref().unwrap_or_default();
    assert!(
        help.contains("`(x + 1)!`") && help.contains("`(x + 1) ?? d`"),
        "{help}"
    );
    let d = check("let xs: List<?Int> = [1, None]\nprint(xs)\n");
    let help = d[0].help.as_deref().unwrap_or_default();
    assert!(help.contains("`print(xs, none: d)`"), "{d:?}");
}

#[test]
fn the_old_optional_spellings_are_removed_with_a_fix_and_none_stays_a_name() {
    let d = check(
        "let a: ?Int = some(3)\nlet b: ?Int = none\nif let some(y) = b: print(y)\nprint(a ?? 0)\n",
    );
    let fixed: Vec<_> = d
        .iter()
        .filter(|d| d.code == cpl_diag::codes::REMOVED_SYNTAX)
        .map(|d| d.fixes[0].edits[0].1.as_str())
        .collect();
    assert_eq!(fixed, ["Some", "None", "Some"], "{d:?}");
    for named in [
        "let none: List<Int> = []\nprint(none.len)\n",
        "fn f(some: Int) -> Int = some + 1\nprint(f(1))\n",
        "fn g() -> ?Int:\n    let none = 2\n    return None\nprint(g() ?? 0)\n",
    ] {
        assert!(check(named).is_empty(), "{named}: {:?}", check(named));
    }
}

#[test]
fn walking_a_type_alias_walks_the_aliased_type() {
    let src = "domain R = 0..3\ntype G = Subset<R>\nvar s: [G]Int = 0\nfor g in G: s[g] = g.len\nprint(s[G.empty])\n";
    assert!(check(src).is_empty(), "{:?}", check(src));
}

#[test]
fn is_empty_on_a_type_with_len_is_fixed_to_len_zero() {
    let d = check("var w = Window(+)\nw.push(3)\nif not w.is_empty: print(1)\n");
    assert_eq!(d.len(), 1, "{d:?}");
    assert_eq!(d[0].fixes[0].edits[0].1, "len == 0", "{d:?}");
}

#[test]
fn an_uppercase_value_is_renamed_with_every_use() {
    let src = "let K: 1..=9 = read()\n\
               domain Slot = 0..K\n\
               fn f() -> Int = K - 1\n\
               print(f(), Slot.size, K)\n";
    let d = check(src);
    assert_eq!(d.len(), 1, "{d:?}");
    let edits = &d[0].fixes.last().unwrap().edits;
    assert_eq!(edits.len(), 4, "{edits:?}");
    let mut fixed = src.to_string();
    for (span, text) in edits.iter().rev() {
        fixed.replace_range(span.start as usize..span.end as usize, text);
    }
    assert!(check(&fixed).is_empty(), "{fixed}: {:?}", check(&fixed));
    let taken = check("let K = 3\nlet k = K + 1\nprint(k)\n");
    assert!(
        taken[0].fixes.iter().all(|f| f.edits.len() < 2),
        "{taken:?}"
    );
}

#[test]
fn a_pair_over_runs_gets_each_value_and_its_count() {
    for src in [
        "let xs = [1, 1, 2]\nfor (x, n) in runs(xs): print(x, n)\n",
        "let xs = [1, 1, 2]\nfor (x, n) in xs.runs(): print(x, n)\n",
    ] {
        let d = check(src);
        assert_eq!(d.len(), 1, "{d:?}");
        let (span, text) = &d[0].fixes[0].edits[0];
        let mut fixed = src.to_string();
        fixed.replace_range(span.start as usize..span.end as usize, text);
        assert!(check(&fixed).is_empty(), "{fixed}: {:?}", check(&fixed));
    }
}

#[test]
fn summing_a_range_is_fixed_to_fold() {
    let src = "use std.fenwick.{Fenwick}\n\
               domain P = 0..5\n\
               var f = Fenwick<P, Int>.new()\n\
               for p in P: f.add(p, 1)\n\
               let l = P(1)\n\
               print(f.sum(l..=l))\n";
    let d = check(src);
    assert_eq!(d.len(), 1, "{d:?}");
    assert_eq!(d[0].fixes[0].edits[0].1, "fold", "{d:?}");
    let fixed = src.replace(".sum(", ".fold(");
    assert!(check(&fixed).is_empty(), "{:?}", check(&fixed));
    let point = check(&src.replace("l..=l", "l"));
    assert!(
        point
            .iter()
            .all(|d| d.fixes.iter().all(|f| f.edits[0].1 != "fold")),
        "{point:?}"
    );
}

#[test]
fn an_uncalled_method_is_called() {
    let head = "struct Z(v: Int)\n\
                impl Mul for Z:\n\
                \x20   fn mul(self, o: Z) -> Z = Z(self.v * o.v)\n\
                impl Z:\n\
                \x20   fn inv(self) -> Z = Z(self.v)\n\
                let a = Z(3)\n";
    for (body, fixed) in [
        (
            "let inv = a.inv\nprint((a * inv).v)\n",
            "let inv = a.inv()\nprint((a * inv).v)\n",
        ),
        ("print((a * a.inv).v)\n", "print((a * a.inv()).v)\n"),
    ] {
        let d = check(&format!("{head}{body}"));
        let calls = d
            .iter()
            .flat_map(|d| &d.fixes)
            .filter(|f| f.edits[0].1 == "()");
        assert!(calls.count() >= 1, "{d:?}");
        let clean = check(&format!("{head}{fixed}"));
        assert!(clean.is_empty(), "{clean:?}");
    }
}

#[test]
fn a_builtin_that_may_give_nothing_is_optional() {
    let d = check("var xs: List<Int> = []\nprint(xs.pop())\n");
    assert_eq!(d.len(), 1, "{d:?}");
    assert!(d[0].message.contains("`xs.pop()` has type `?Int`"), "{d:?}");
    assert!(
        d[0].help.as_deref().unwrap_or("").contains("xs.pop()!"),
        "{d:?}"
    );
    let fix = &d[0].fixes[0].edits;
    assert_eq!(fix[1].1, "!", "{d:?}");
    let d = check("var s = \"ab\"\nlet c: Char = s.pop()\n");
    assert!(d.iter().any(|d| d.message.contains("?Char")), "{d:?}");
    let clean = check(
        "var xs = [5, 6]\nlet k = xs.find(x => x > 5)!\nwhile let u = xs.pop():\n    print(u + k)\n",
    );
    assert!(clean.is_empty(), "{clean:?}");
}

#[test]
fn a_list_getter_that_may_give_nothing_is_optional() {
    let d = check("var xs: List<Int> = []\nprint(xs.first)\n");
    assert_eq!(d.len(), 1, "{d:?}");
    assert!(d[0].message.contains("`xs.first` has type `?Int`"), "{d:?}");
    let clean = check("var xs = [5, 6]\nprint(xs.first!, xs.last ?? 0)\n");
    assert!(clean.is_empty(), "{clean:?}");
}

#[test]
fn a_name_holding_an_optional_position_may_be_none_not_step() {
    let d = check(
        "domain V = 0..3\nvar a: [V]Int = [0, 0, 0]\nvar r: List<V> = [V.first!]\nlet u = r.pop()\na[u] += 1\n",
    );
    assert_eq!(d.len(), 1, "{d:?}");
    assert!(
        d[0].message.contains("`u` is a ?V: it may be `None`"),
        "{d:?}"
    );
    assert!(
        d[0].labels
            .iter()
            .any(|l| l.message.contains("declared as a ?V")),
        "{d:?}"
    );
    let help = d[0].help.as_deref().unwrap_or("");
    assert!(help.contains("`u!`") && !help.contains("edge"), "{help}");
}

#[test]
fn rubric_batch_names_the_optional_the_with_and_the_part() {
    let d = check("let a: ?Int = 1\nlet b: ?Int = 2\nif a < b:\n    print(1)\n");
    assert!(d[0].message.contains("`a` is an ?Int"), "{d:?}");
    assert!(
        d[0].help.as_deref().unwrap_or("").contains("`a! < b!`"),
        "{d:?}"
    );
    let d = check("print(1)\nwith io(sep: \",\")\nprint(1, 2)\n");
    assert!(
        d[0].help
            .as_deref()
            .unwrap_or("")
            .contains("block's first line"),
        "{d:?}"
    );
    let d = check("var xs = [1, 2]\nxs[0] = xs.pop()!\nprint(xs)\n");
    assert!(d[0].message.ends_with("`xs[0]` is part of `xs`"), "{d:?}");
    let help = d[0].help.as_deref().unwrap_or("");
    assert!(
        help.contains("`let v = xs.pop()!`, then `xs[0] = v`"),
        "{help}"
    );
}

#[test]
fn a_header_field_is_made_var_in_a_block_struct() {
    let d = check("struct P(x: Int)\nvar p = P(1)\np.x = 2\n");
    assert_eq!(d.len(), 1, "{d:?}");
    assert!(d[0].fixes.is_empty(), "{d:?}");
    let help = d[0].help.as_deref().unwrap_or("");
    assert!(help.contains("block `struct P:`"), "{help}");
}

#[test]
fn the_old_copy_spelling_is_one_error_with_the_clone_fix() {
    for src in [
        "let xs = [1, 2]\nlet ys = xs.copy()\nprint(ys)\n",
        "let s = \"ab\"\nlet t = s.copy()\nprint(t)\n",
        "var m = Map<Int, Int>()\nlet n = m.copy()\nprint(n.len)\n",
    ] {
        let d = check(src);
        assert_eq!(d.len(), 1, "{src}: {d:?}");
        assert!(d[0].message.contains("`.clone()`"), "{d:?}");
        assert_eq!(d[0].fixes[0].edits[0].1, "clone", "{d:?}");
    }
    assert!(check("let xs = [1, 2]\nlet ys = xs.clone()\nprint(ys)\n").is_empty());
}

#[test]
fn a_free_function_called_as_a_method_gets_the_call_spelling() {
    let src = "let mask = 5\nprint(mask.popcount())\n";
    let d = check(src);
    assert_eq!(d.len(), 1, "{d:?}");
    assert!(
        d[0].help
            .as_deref()
            .unwrap_or("")
            .contains("`popcount(mask)`"),
        "{d:?}"
    );
    let fix = &d[0].fixes[0].edits[0];
    let mut fixed = src.to_string();
    fixed.replace_range(fix.0.start as usize..fix.0.end as usize, &fix.1);
    assert_eq!(fixed, "let mask = 5\nprint(popcount(mask))\n");
    assert!(check(&fixed).is_empty(), "{fixed}");
}

#[test]
fn a_name_stopped_short_means_the_prelude_name_before_a_std_typo() {
    let d = check("pri\n");
    assert_eq!(d.len(), 1, "{d:?}");
    assert_eq!(d[0].help.as_deref(), Some("did you mean `print`?"), "{d:?}");
}

#[test]
fn a_generic_alias_binds_its_parameters() {
    let src = "use std.seq.{Perm}\ndomain V = 1..=3\nlet p: Perm<V> = read()\nprint(p[V(1)])\n";
    assert!(check(src).is_empty(), "{:?}", check(src));
}

#[test]
fn a_getter_called_with_parens_offers_to_drop_them() {
    let d = one("let xs = [3, 1]\nprint(xs.len())\n");
    assert_eq!(d.code, cpl_diag::codes::NOT_CALLABLE);
    assert_eq!(
        fixed("let xs = [3, 1]\nprint(xs.len())\n"),
        "let xs = [3, 1]\nprint(xs.len)\n"
    );
    assert_eq!(
        fixed("let xs = [3, 1]\nprint(xs.first())\n"),
        "let xs = [3, 1]\nprint(xs.first)\n"
    );
}

#[test]
fn a_method_no_fn_in_reach_takes_the_receiver_is_missing_not_a_mismatch() {
    let d = one("var a: [3]Int = [1, 2, 3]\na.push(4)\n");
    assert_eq!(d.code, cpl_diag::codes::UNKNOWN_FIELD, "{}", d.message);
    assert_eq!(d.message, "a `[3]Int` has no method `push`");
    assert!(
        d.help
            .as_deref()
            .is_some_and(|h| h.contains("`Counter<K, Int>`")),
        "{:?}",
        d.help
    );
    assert!(check("var xs = [1, 2]\nxs.push(3)\nprint(xs.len)\n").is_empty());
}

#[test]
fn a_bare_variant_two_enums_declare_needs_an_expected_type() {
    let d = one("enum Side = Left | Right\nenum Turn = Left | Back\nlet s = Left\n");
    assert_eq!(d.code, cpl_diag::codes::UNRESOLVED_NAME, "{}", d.message);
    assert!(
        d.message.contains("`Side`") && d.message.contains("`Turn`"),
        "{}",
        d.message
    );
    let d = one("enum Side = Left | Right\nenum Turn = Left | Back\nlet s = if true: (1, Left) else: (2, Right)\n");
    assert_eq!(d.code, cpl_diag::codes::UNRESOLVED_NAME, "{}", d.message);
    for ok in [
        "enum Side = Left | Right\nenum Turn = Left | Back\nlet s = Side.Left\nlet t = Back\nprint(yes(s == Side.Right), yes(t == Turn.Back))\n",
        "enum Side = Left | Right\nlet s = Left\nprint(yes(s == Right))\n",
    ] {
        assert!(check(ok).is_empty(), "{ok}: {:?}", check(ok));
    }
}

#[test]
fn a_typo_is_never_offered_a_fn_whose_self_is_another_type() {
    let d = one("let a = [3, 1, 2]\nprint(a.hlve())\n");
    assert_eq!(d.code.0, "E0203");
    assert!(
        !d.help.as_deref().unwrap_or("").contains("halve"),
        "{:?}",
        d.help
    );
}

#[test]
fn a_missing_member_with_no_close_name_lists_what_the_type_has() {
    let src = "struct P:\n    x: Int\n    fn up(self) -> Int = self.x + 1\nlet p = P(1)\nprint(p.down())\n";
    let d = one(src);
    assert_eq!(d.code.0, "E0203");
    assert_eq!(d.help.as_deref(), Some("its fields are `x`"));
    let d = one("let s = scan([3, 1, 2], +)\nprint(s.min())\n");
    let help = d.help.unwrap_or_default();
    assert!(help.starts_with("its methods are `all`, `at`"), "{help}");
    assert!(
        !help.contains("`new`") && !help.contains("`filling`"),
        "{help}"
    );
}

#[test]
fn a_comprehension_over_a_range_is_not_called_a_list_of_one_range() {
    let d = one("let n = 3\nprint([i for i in 1..=n].shuffled())\n");
    let help = d.help.unwrap_or_default();
    assert!(!help.contains("one range"), "{help}");
    assert!(
        help.contains("`var ys = [i for i in 1..=n].clone()`"),
        "{help}"
    );
    let d = one("let n = 3\nprint([1..=n].shuffled())\n");
    assert!(d.help.unwrap_or_default().contains("holding one range"));
}

#[test]
fn unwrapping_an_array_or_list_of_optionals_unwraps_every_cell() {
    let src = "fn f(n: Int) -> Int:\n    var at: [0..n]?Int = None\n    let at = at!\n    at[0] + 1\nvar xs: List<?Int> = []\nlet ys: List<Int> = xs!\nprint(f(1), ys.len)\n";
    assert!(check(src).is_empty(), "{:?}", check(src));
}

#[test]
fn a_param_the_body_changes_gets_one_error_not_one_more_at_the_call() {
    let d = one("fn f(a: List<Int>):\n    a.push(1)\nvar b = [1]\nf(mut b)\n");
    assert_eq!(d.code.0, "E0403");
    assert!(d.help.unwrap_or_default().contains("write `mut a`"));
}

#[test]
fn a_top_level_unwrap_rebinds_its_name_and_other_redefinitions_stay_e0202() {
    let src =
        "var at: [0..3]?Int = None\nfor i in 0..3: at[i] = i\nlet at = at!\nprint(at[2] + 1)\n";
    assert!(check(src).is_empty(), "{:?}", check(src));
    let d = one("let a = 1\nlet a = 2\nprint(a)\n");
    assert_eq!(d.code.0, "E0202");
}

#[test]
fn a_call_with_type_arguments_is_called_so_it_gets_no_parens_warning() {
    let called = check("let s = \"12\"\nprint(s.parse<Int>()! + 1)\n");
    assert!(called.iter().all(|d| d.code.0 != "L0608"), "{called:?}");
    let bare = check("let a = [2, 1]\nprint(a.sorted)\n");
    assert!(bare.iter().any(|d| d.code.0 == "L0608"), "{bare:?}");
}

#[test]
fn a_bare_variant_two_enums_declare_reads_as_the_expected_enum() {
    let src = "enum Turn = Left | Back\nenum Side = Left | Right\nfn f() -> Turn = Left\nlet t: Turn = if true: Left else: Back\nlet s: List<Side> = [Left, Right]\nprint(f(), t, s.len)\n";
    assert!(check(src).is_empty(), "{:?}", check(src));
    let d =
        one("enum Turn = Left | Back\nenum Side = Left | Right\nlet t: Turn = Right\nprint(t)\n");
    assert_eq!(d.code.0, "E0302");
}
