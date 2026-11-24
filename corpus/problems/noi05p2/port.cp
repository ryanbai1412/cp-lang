// Source: NOI 2005 P2 (Sequence Maintenance), https://dmoj.ca/problem/noi05p2
// limits: 600 ms, 64 MB
// Problem: a sequence of N ints in [-1000, 1000] and M <= 20000 ops: INSERT tot numbers after
//   posi; DELETE, MAKE-SAME c, REVERSE, GET-SUM (tot may be 0) the tot numbers from posi; MAX-SUM
//   (best non-empty subarray). Print each query; <= 5e5 numbers alive, <= 4e6 inserted in all.
// Solution: a std `SeqFold` keeps the numbers themselves and folds each as a `MaxSub` summary
//   (best run, total, best prefix and suffix; `lift: MaxSub.of`), under an assign action whose
//   tag on a run of len equal numbers is `MaxSub.of(c).repeat(len)`; `mirror:` reverses a fold.
//   Each op is one range call on positions posi - 1 .. posi - 1 + tot. O(tot + log n) expected
//   per op; erased nodes are reused, so memory follows the live length.
// Bounds: n: 1..=5e5; m: 1..=20_000; every number: -1000..=1000; posi..posi + tot stays inside
//   the sequence, which is never empty
// original: corpus/submissions/7249409.cpp

use std.algebra.{MaxSub}
use std.lazy.{SeqFold}

type Val = -1000..=1000
let assign = Action<MaxSub, ?Val>.new(
    id: None,
    compose: (g, f) => g ?? f,
    apply: (f, x, len) => if let c = f: MaxSub.of(c).repeat(len) else: x,
)

let (n, m): (1..=500_000, 1..=20_000) = read()
domain Init = 0..n
var seq = SeqFold<MaxSub, ?Val, Val>(+, read<[Init]Val>(), act: assign, mirror: x => x.reversed(), lift: MaxSub.of)
repeat m:
    let op: Str = read()
    if op == "MAX-SUM":
        print(seq.total.best)
        continue
    let (posi, tot): (0..=seq.len, 0..=seq.len + 4_000_000) = read()
    let r = posi - 1..posi - 1 + tot
    match op:
        "INSERT" => seq.insert_many(posi, read<[tot]Val>())
        "DELETE" => seq.erase_range(r)
        "MAKE-SAME" => seq.apply(r, Some(read<Val>()))
        "REVERSE" => seq.reverse(r)
        _ => print(seq.fold(r).total)
