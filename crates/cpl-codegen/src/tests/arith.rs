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
        body.contains("cpl::put_int(ll(uint32_t(a) / uint32_t(b)))"),
        "{body}"
    );
    assert!(
        body.contains("cpl::put_int(ll(uint32_t(a) % uint32_t(b)))"),
        "{body}"
    );
    assert!(body.contains("cpl::put_int(a % 1000000007)"), "{body}");
    assert!(
        !body.contains("cpl::ediv") && !body.contains("cpl::divp"),
        "{body}"
    );
}

#[test]
fn a_divisor_unpacked_from_input_keeps_its_declared_range() {
    let src = "let item: [0..3](cost: 1..=200, worth: Int) = read()\nlet (c, w) = item[1]\nlet a: 0..=100 = read()\nprint(a / c, a % c, w)\n";
    let body = body(src);
    assert!(body.contains("(a / c)"), "{body}");
    assert!(body.contains("(a % c)"), "{body}");
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
    assert!(body.contains("cpl::put_int(i % 3)"), "{body}");
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
        mul.contains("% 998244353") && !mul.contains("cpl::modp"),
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
        body.contains("s % 1000") && body.contains("cpl::modp(x, 1000)"),
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
    assert!(cpp.contains("cpl::imul(x, 3), 1) % m;"), "{cpp}");
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
