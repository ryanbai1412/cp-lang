//! A library-domain walk is spelled out in lowering: no `Iter` can hold one.

use crate::tests::lower;

#[test]
fn no_library_domain_walk_survives_lowering() {
    let program = lower(concat!(
        "struct Sub<B>(bits: Int)\n",
        "impl Index for Sub<B>:\n",
        "    let count = 1 << B.count\n",
        "    @get fn ord(self) -> Int = bits\n",
        "    fn nth(k: Int) -> Sub<B> = Sub(k)\n",
        "domain V = 0..3\n",
        "fn total<D: Domain>() -> Int = Sub<D>.count(s => s.bits > 1)\n",
        "for s in Sub<V>.rev: print(s.bits)\n",
        "print(Sub<V>.find(s => s.bits > 2).map(s => s.bits) ?? -1, total<V>())\n",
        "print([s.bits for s in Sub<V>], Sub<V>.min(by: s => -s.bits)!.bits)\n",
        "let any6 = () => Sub<V>.any(s => s.bits == 6)\n",
        "print([s.bits for s in Sub<V>.rev if s.bits > 3], any6())\n",
    ));
    // An `Iter` only lives in a `Stmt::For` head, but scan the whole program
    // (fused loops, comprehensions, closures, tables) so a Walk anywhere fails.
    let dump = format!("{program:?}");
    assert!(
        !dump.contains("Walk {"),
        "an Iter::Walk reached the backends"
    );
    assert!(
        dump.contains("Range {"),
        "the walks should expand to ordinal ranges"
    );
}

#[test]
fn print_works_out_a_walk_ahead_of_a_later_call() {
    // The walk calls `nth`, so its count is worked out before the line is
    // written, as any argument that calls a fn is (R10.3).
    let out = crate::tests::cpp(concat!(
        "struct Sub<B>(bits: Int)\n",
        "impl Index for Sub<B>:\n",
        "    let count = 1 << B.count\n",
        "    @get fn ord(self) -> Int = bits\n",
        "    fn nth(k: Int) -> Sub<B> = Sub(k)\n",
        "domain V = 0..3\n",
        "print(Sub<V>.count(s => s.bits > 1), 7)\n",
    ));
    assert!(!out.contains("put_int([&]"), "{out}");
    assert!(
        out.contains("t3 += 1;\n    }\n  }\n  ll t4 = t3;\n  cpl::put_int(t4, ' ');"),
        "{out}"
    );
}
