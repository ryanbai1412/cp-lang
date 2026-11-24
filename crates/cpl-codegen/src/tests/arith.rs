//! Division, remainders and the signs and ranges that make them plain C++.

use super::{body, cpp};

#[test]
fn filter_map_sum_fuses_without_intermediate_vectors() {
    let body = body(
        "let a: List<Int> = [1, 2, 3, 4]\nprint(a.filter(x => (x & 1) == 0).map(x => x * 3 + 1).sum())\n",
    );
    assert!(
        !body.contains("push_back"),
        "filter.map.sum should fuse into one loop without intermediate vectors:\n{body}"
    );
}

#[test]
fn power_of_two_div_and_rem_use_shift_and_mask() {
    let body =
        body("var a: Int = read()\nprint(a / 2, a % 2, a / 8, a % 8)\na /= 2\na %= 4\nprint(a)\n");
    assert!(body.contains("a >> 1"), "{body}");
    assert!(body.contains("a & 1"), "{body}");
    assert!(body.contains("a >> 3"), "{body}");
    assert!(body.contains("a & 7"), "{body}");
    assert!(body.contains("a >>= 1;"), "{body}");
    assert!(body.contains("a &= 3;"), "{body}");
    assert!(
        !body.contains("cpl::divp") && !body.contains("cpl::modp"),
        "{body}"
    );
}

#[test]
fn division_by_a_positive_number_of_a_non_negative_one_is_plain_cpp() {
    let body = body(
        "let a: 0..=100 = read()\nlet b: 1..=100 = read()\nprint(a / b, a % b, a % 1000000007)\n",
    );
    assert!(
        body.contains("cpl::put_int(ll(uint32_t(a) / uint32_t(b))"),
        "{body}"
    );
    assert!(
        body.contains("cpl::put_int(ll(uint32_t(a) % uint32_t(b))"),
        "{body}"
    );
    // Below the modulus already: nothing to reduce.
    assert!(body.contains("cpl::put_int(a, '\\n')"), "{body}");
    assert!(
        !body.contains("cpl::ediv") && !body.contains("cpl::divp"),
        "{body}"
    );
}

#[test]
fn a_divisor_unpacked_from_input_keeps_its_declared_range() {
    let src = "let item: [0..3](cost: 1..=200, worth: Int) = read()\nlet (c, w) = item[1]\nlet a: 0..=100 = read()\nprint(a / c, a % c, w)\n";
    let body = body(src);
    assert!(body.contains("put_int(a / c, "), "{body}");
    assert!(body.contains("put_int(a % c, "), "{body}");
}

#[test]
fn labels_of_a_positive_domain_divide_in_32_bits() {
    let body = body(concat!(
        "domain Val = 1..=200000\n",
        "let a: [3]Val = read()\n",
        "print(a[0].label % a[1].label, a[0].label / a[2].label)\n",
    ));
    assert!(body.contains("% uint32_t("), "{body}");
    assert!(body.contains("/ uint32_t("), "{body}");
    assert!(
        !body.contains("cpl::emod(") && !body.contains("cpl::ediv("),
        "{body}"
    );
}

#[test]
fn a_wide_non_negative_dividend_divides_unsigned() {
    let body = body("let a: 0..=1000000000 = read()\nlet b: 1..=100 = read()\nprint(a * a % 998244353, a * a / b)\n");
    assert!(body.contains("ll(uint64_t(a * a) % 998244353)"), "{body}");
    assert!(body.contains("ll(uint64_t(a * a) / uint64_t(b))"), "{body}");
}

#[test]
fn a_dividend_that_may_be_negative_gets_the_branchless_fix() {
    let source = "let a: Int = read()\nlet b: 1..=100 = read()\nprint(a / b, a % b)\n";
    let body = body(source);
    assert!(
        body.contains("cpl::divp(a, b)") && body.contains("cpl::modp(a, b)"),
        "{body}"
    );
    let cpp = cpp(source);
    assert!(
        cpp.contains("inline ll modp(ll a, ll b)") && cpp.contains("return r + ((r >> 63) & b);"),
        "{cpp}"
    );
    // A dividend that is non-negative at run time skips the signed remainder and its fix-up.
    assert!(
        cpp.contains("if (__builtin_expect(a >= 0, 1)) return ll(uint64_t(a) % uint64_t(b));"),
        "{cpp}"
    );
    // The helpers it does not call are not in the file.
    assert!(!cpp.contains("inline ll emod"), "{cpp}");
}

#[test]
fn a_divisor_of_unknown_sign_gets_the_full_euclidean_helpers() {
    let body = body("let a: Int = read()\nlet b: Int = read()\nprint(a / b, a % b)\n");
    assert!(
        body.contains("cpl::ediv(a, b)") && body.contains("cpl::emod(a, b)"),
        "{body}"
    );
}

#[test]
fn a_variable_assigned_later_is_not_trusted_to_keep_its_range() {
    let body = body("var a: 0..=100 = read()\na -= 500\nprint(a % 7)\n");
    assert!(body.contains("cpl::modp(a, 7)"), "{body}");
}

#[test]
fn loop_counters_and_lengths_are_known_non_negative() {
    let body =
        body("let xs: List<Int> = [1, 2, 3]\nfor i in 0..10:\n    print(i % 3, xs.len / 2)\n");
    assert!(body.contains("cpl::put_int(i % 3"), "{body}");
    assert!(body.contains("(ll)xs.size() >> 1"), "{body}");
}

#[test]
fn u32_struct_field_proves_nonneg_for_mod_mul() {
    let out = cpp("use std.mod.{Mod}
type Z = Mod<998_244_353>
let a: Int = read()
let b: Int = read()
print((Z(a) * Z(b)).v)
");
    let mul = out
        .split("\ninline ")
        .find(|s| s.contains("_mul("))
        .expect("Mod::mul emitted");
    assert!(
        mul.contains(") % 998244353") && mul.contains("uint64_t(") && !mul.contains("cpl::modp"),
        "U32-narrowed Mod.v fields should prove non-negativity and avoid cpl::modp:\n{mul}"
    );
}
#[test]
fn multi_store_nonneg_var_drops_modp_and_sub_taints() {
    let out = cpp("let n: 1..=100 = read()
var s = 123
var x = 10
var acc = 0
for _ in 0..n:
    s = (s * 1103515245 + 12345) & 2147483647
    x -= 1
    acc += s % 1000 + x % 1000
print(acc)
");
    let body = out.split("void solve()").nth(1).expect("solve emitted");
    assert!(
        body.contains(" s % 1000")
            && !body.contains("modp(s")
            && body.contains("cpl::modp(x, 1000)"),
        "bitmask-updated var should prove NonNeg while sub-updated var stays tainted:\n{body}"
    );
}

#[test]
fn a_guarded_step_of_a_global_counter_does_not_branch() {
    let cpp = cpp(
        "let a: [0..8]Int = read()\nvar seen: [10]Int = 0\nvar distinct = 0\n\
        fn add(i: Int):\n    if seen[a[i]] == 0: distinct += 1\n    seen[a[i]] += 1\n\
        fn rem(i: Int):\n    seen[a[i]] -= 1\n    if seen[a[i]] == 0: distinct -= 1\n\
        for i in 0..8: add(i)\nrem(0)\nprint(distinct)\n",
    );
    assert!(
        cpp.contains("distinct += (") && cpp.contains(") ? 1 : 0;"),
        "{cpp}"
    );
    assert!(cpp.contains("distinct -= ("), "{cpp}");
    assert!(!cpp.contains("distinct += 1;"), "{cpp}");
}

#[test]
fn a_ranged_param_divides_with_cpps_own_operator() {
    let cpp = cpp(concat!(
        "fn step(x: 0..=1000, m: 1..=1000) -> Int:\n",
        "    (x * 3 + 1) % m\n",
        "let n: Int = read()\n",
        "print(step(n, 7))\n",
    ));
    assert!(
        cpp.contains("ll(uint32_t(x * 3 + 1) % uint32_t(m));"),
        "{cpp}"
    );
    assert!(!cpp.contains("cpl::emod("), "{cpp}");
    // Every call checks its argument into the range the body relies on.
    assert!(cpp.contains("step(cpl::check_range(n, 0, 1000,"), "{cpp}");
}

/// A domain position's ord is never negative, so `/ 64` and `% 64` on it
/// are a shift and a mask, and a shift by the mask needs no `shamt`.
#[test]
fn a_bit_index_from_an_ord_is_a_shift_and_a_mask() {
    let out = cpp(
        "domain D = 0..=1000\nlet b = Bitset<D>.single(D(4))\nlet i: 0..=1000 = read()\nprint(yes(b.contains(D(i))))\n",
    );
    let body = out
        .split("_contains(const ")
        .nth(1)
        .expect("contains is emitted");
    let line = body.lines().nth(1).expect("its body");
    assert!(line.contains(" >> 6)") && line.contains(" & 63)"), "{line}");
    assert!(!line.contains("shamt") && !line.contains("/ 64"), "{line}");
}

/// An amount proven in `0..64` shifts without `shamt`; an unproven one keeps it.
#[test]
fn a_shift_by_a_proven_amount_has_no_shamt() {
    let out = cpp("let k: Int = read()\nvar x = 1\nx <<= k % 64\nprint(x >> (k & 63), x << k)\n");
    assert!(out.contains("x = cpl::ishl(x, k & 63);"), "{out}");
    assert_eq!(out.matches("cpl::shamt(").count(), 1, "{out}");
}

#[test]
fn a_branch_on_a_const_generic_against_a_named_const_folds() {
    let out = cpp(concat!(
        "const Q = 998_244_353\n",
        "fn root<P>(x: Mod<P>) -> Int:\n",
        "    if P != Q - 0: return x.v * 7 + 1\n",
        "    3\n",
        "print(root(Mod<998_244_353>(5)))\n",
    ));
    let root = &out[out.find("inline ll root_").expect("root")..];
    let root = &root[..root.find("\n}").expect("root's end")];
    assert!(root.ends_with("return 3;"), "{root}");
    assert!(!root.contains("if ("), "{root}");
}

#[test]
fn ntt_over_the_first_ntt_prime_leaves_primitive_root_out() {
    let out = cpp(concat!(
        "use std.ntt.{convolve}\n",
        "let a = [Mod<998_244_353>(x) for x in [1, 2, 3]]\n",
        "let b = [Mod<998_244_353>(x) for x in [4, 5]]\n",
        "print([c.v for c in convolve(a, b)])\n",
    ));
    assert!(out.contains("ntt"), "{out}");
    for gone in ["primitive_root", "factor", "rho"] {
        assert!(!out.contains(gone), "{gone}: {out}");
    }
}

#[test]
fn a_grid_index_folds_its_dims_with_no_loop() {
    let out = cpp("domain A = [3]\ndomain B = [4]\nvar g: [A, B, A]Int = 0\nlet i: 0..3 = read()\ng[A(i), B(0), A(i)] = 1\nprint(g[A(i), B(0), A(i)])\n");
    assert!(
        out.contains("((p = p * n[k++] + (size_t)(ll)i), ...)"),
        "{out}"
    );
    assert!(!out.contains("for (int k = 0; k < K; k++) p ="), "{out}");
}

#[test]
fn a_non_negative_dividend_drops_the_sign_fix_up_whatever_the_divisor() {
    let body = body("let a: 0..=1000000 = read()\nlet b: Int = read()\nprint(a / b, a % b)\n");
    assert!(body.contains("cpl::divn(a, b)"), "{body}");
    assert!(body.contains("cpl::modn(a, b)"), "{body}");
    assert!(
        !body.contains("cpl::ediv") && !body.contains("cpl::emod"),
        "{body}"
    );
}

#[test]
fn convolve_past_the_ntt_primes_keeps_their_residues_off_the_halves_path() {
    let out = cpp(concat!(
        "use std.ntt.{convolve}\n",
        "let a = [Mod<1_000_000_007>(x) for x in [1, 2, 3]]\n",
        "print([c.v for c in convolve(a, a)])\n",
    ));
    assert!(out.contains("crt_product"), "{out}");
    assert!(!out.contains("halves_product"), "{out}");
}

#[test]
fn convolve_over_an_ntt_prime_leaves_the_fft_and_crt_paths_out() {
    let out = cpp(concat!(
        "use std.ntt.{convolve}\n",
        "let a = [Mod<998_244_353>(x) for x in [1, 2, 3]]\n",
        "print([c.v for c in convolve(a, a)])\n",
    ));
    assert!(out.contains("halves_product"), "{out}");
    assert!(!out.contains("split_product"), "{out}");
    assert!(!out.contains("crt_product"), "{out}");
}

#[test]
fn a_guarded_step_by_more_than_one_keeps_its_overflow_check() {
    let cpp = cpp("let xs: [0..4]Int = read()\nvar h = 0\n\
         fn f():\n    for x in xs:\n        if x > 5: h += 1000000000000000000\n\
         f()\nprint(h)\n");
    assert!(cpp.contains("cpl::iadd(h, 1000000000000000000"), "{cpp}");
    assert!(!cpp.contains("h += ("), "{cpp}");
}

#[test]
fn a_restored_variable_is_not_its_first_value() {
    let cpp = cpp(
        "let n: 1..=10 = read()\nvar c = 0\nfor i in 0..n:\n    c = c + 1000000000000000000\nprint(c)\n",
    );
    assert!(cpp.contains("cpl::iadd(c, 1000000000000000000"), "{cpp}");
}

#[test]
fn a_wrap_reduces_once_by_how_far_its_value_can_lap() {
    let src = |step: &str| {
        format!(
            "let d: 1..=100 = read()\ndomain Rem = 0..d\nvar t: [Rem]Int = 0\n\
             for r in Rem:\n    for x in 0..=9: t[Rem.wrap(r.ord + {step})] += 1\nprint(t[Rem(0)])\n"
        )
    };
    // Up to nine past the count: one remainder, in 32 bits, and no position check.
    let body = body(&src("x"));
    let far = body
        .lines()
        .find(|l| l.contains("t1 ="))
        .expect("the wrapped index");
    assert_eq!(far.matches('%').count(), 1, "{far}");
    for helper in [
        "cpl::emod",
        "cpl::modn",
        "cpl::modp",
        "cpl::pos(",
        "cpl::iadd",
    ] {
        assert!(!far.contains(helper), "{helper}: {far}");
    }
    // Under two laps: a conditional subtract, no division.
    let near = super::body(&src("1"));
    assert!(
        near.contains("cpl::sub_if(") && !near.contains('%'),
        "{near}"
    );
}

#[test]
fn bounded_operands_add_without_an_overflow_check_whatever_their_sign() {
    let out = cpp("use std.mod.{Mod}\ntype Z = Mod<1_000_000_007>\nlet a: Int = read()\nprint((Z(a) + Z(a)).v)\n");
    let add = out
        .split("\ninline ")
        .find(|s| s.contains("_add(const"))
        .expect("Mod::add emitted");
    assert!(!add.contains("cpl::iadd"), "{add}");
}

#[test]
fn a_residue_sum_of_products_reduces_once() {
    let body = body(concat!(
        "use std.mod.{Mod}\ntype Z = Mod<1_000_000_007>\nlet n: 1..=100 = read()\n",
        "let a: List<Z> = [Z(read<Int>()) for _ in 0..n]\nvar s = Z(0)\n",
        "for x in a:\n    for y in a: s = s + x * y\nprint(s)\n",
    ));
    let line = body
        .lines()
        .find(|l| l.contains("s = ") && !l.contains("(0)"))
        .expect("the sum");
    assert!(!line.contains("_add(") && !line.contains("_mul("), "{line}");
    assert_eq!(line.matches("% 1000000007").count(), 1, "{line}");
}

#[test]
fn a_chain_of_residue_adds_over_a_product_and_a_new_reduces_once() {
    let body = body(concat!(
        "use std.mod.{Mod}\ntype Z = Mod<998_244_353>\nlet n: 1..=100 = read()\n",
        "let a: List<Int> = [read() for _ in 0..n]\nvar acc = Z.new(1)\nlet c = Z.new(31)\n",
        "let shift = Z.new(n + 1)\nfor i in 0..n: acc = acc * c + Z.new(a[i]) + shift\nprint(acc)\n",
    ));
    let line = body
        .lines()
        .find(|l| l.trim_start().starts_with("acc = ") && !l.contains("_new(1)"))
        .unwrap_or_else(|| panic!("the kernel line:\n{body}"));
    assert!(
        !line.contains("_add(")
            && !line.contains("_mul(")
            && !line.contains("_new(")
            && !line.contains('?'),
        "{line}"
    );
    // The fused `%` and `a[i]`'s own reduction, which its unknown sign keeps.
    assert_eq!(line.matches(" 998244353)").count(), 2, "{line}");
}

#[test]
fn a_residue_matrix_product_sums_unreduced_and_reduces_per_run() {
    let out = cpp("use std.mod.{Mod}
use std.matrix.{Mat}
type Z = Mod<1_000_000_007>
let n: 1..=50 = read()
let a = Mat<Z>.from_flat(n, n, [Z(read()) for _ in 0..n * n])
print((a * a).get(0, 0))
");
    let product = out
        .split("\nstd::optional<")
        .find(|s| s.contains("residue_product") && s.contains("{\n"))
        .expect("residue_product emitted");
    let accumulate = product
        .lines()
        .find(|l| l.contains("row[") && l.contains(" * "))
        .expect("the accumulate line");
    assert!(
        !accumulate.contains("cpl::") && !accumulate.contains('%'),
        "{accumulate}"
    );
    assert_eq!(
        product.matches("cpl::emod").count(),
        1,
        "one reduction per run:\n{product}"
    );
    let mul = out
        .split("\nMat_")
        .find(|s| s.contains("_mul("))
        .expect("Mat.mul emitted");
    assert!(mul.contains("residue_product"), "{mul}");
}

#[test]
fn a_branch_condition_bounds_the_locals_it_compares() {
    let fn_of = |src: &str, name: &str| {
        let out = cpp(src);
        let f = out
            .split("\ninline ")
            .chain(out.split("\nll "))
            .find(|s| s.contains(&format!("{name}(ll")))
            .unwrap_or_else(|| panic!("{name}:\n{out}"))
            .to_string();
        f.split("\n}\n").next().unwrap_or_default().to_string()
    };
    let checked = |f: &str| f.contains("cpl::imul") || f.contains("cpl::iadd");
    // Inside the branch, and past a branch that returns.
    let inside = fn_of(
        "fn f(x: Int, n: Int) -> Int:\n    if x >= 0 and x < 1000 and n >= 0 and n <= 1000: return x * n + 1\n    0\n\
         print(f(read(), read()))\n",
        "f",
    );
    assert!(!checked(&inside), "{inside}");
    let past = fn_of(
        "fn g(x: Int) -> Int:\n    if x < 0 or x > 1000: return 0\n    x * x + x\nprint(g(read()))\n",
        "g",
    );
    assert!(!checked(&past), "{past}");
    // A store after the condition undoes it.
    let stored = fn_of(
        "fn h(x: Int) -> Int:\n    var y = x\n    if y >= 0 and y < 1000:\n        y = read()\n        return y * 2\n    0\n\
         print(h(read()))\n",
        "h",
    );
    assert!(stored.contains("cpl::imul"), "{stored}");
    // So does a store later in a loop body, on the next lap.
    let lapped = fn_of(
        "fn k(x: Int) -> Int:\n    var y = x\n    var s = 0\n    if y >= 0 and y < 1000:\n        for _ in 0..3:\n            s = y * 3\n            y = read()\n    s\n\
         print(k(read()))\n",
        "k",
    );
    assert!(lapped.contains("cpl::imul"), "{lapped}");
    // And a call to a fn that stores to the global.
    let called = body(
        "var g: Int = read()\nfn bump():\n    g = read()\nif g >= 0 and g < 1000:\n    bump()\n    print(g * 2)\n",
    );
    assert!(called.contains("cpl::imul(g, 2)"), "{called}");
}

#[test]
fn a_loop_guard_bounds_what_its_body_sets_the_local_to() {
    let guarded = body(
        "let n: 1..=1_000_000_000 = read()\nvar i = 1\nwhile i < n: i = 2 * i + 1\nprint(i * 2)\n",
    );
    assert!(guarded.contains("i = 2 * i + 1;"), "{guarded}");
    // Each store is at most `2 * (n - 1) + 1`, so `i` stays below 2^31 after the loop.
    assert!(guarded.contains("put_int(i * 2,"), "{guarded}");
    let past = body(
        "let n: 1..=1_000_000_000 = read()\nvar i = 1\nwhile i < n: i = 2 * i + 1\ni = i * i\nprint(i * 2)\n",
    );
    assert!(past.contains("cpl::imul(i, 2)"), "{past}");
}

#[test]
fn a_ranged_parameter_is_bounded_by_its_range() {
    let out = cpp("fn g(n: 0..2_147_483_648) -> Int = n * 2 + 63\nprint(g(read()))\n");
    assert!(out.contains("n * 2 + 63"), "{out}");
    let open = cpp("fn g(n: 0..) -> Int = n + 63\nprint(g(read()))\n");
    assert!(open.contains("cpl::iadd(n, 63)"), "{open}");
}
