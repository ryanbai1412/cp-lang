// Codeforces 2219 B2: Unique Values (Hard Version, interactive).
// In a[1..=2n + 1], one value occurs thrice (at x < y < z) and all others twice.
// Any value occurring c times in S contributes c to |S| and [c == 1] to unique(S),
// which agree mod 2 for c in {0, 1, 2} and disagree only for c = 3.
// Thus (|S| - unique(S)) % 2 == 1 iff {x, y, z} ⊆ S, so three binary searches find z, x, y.

@!interactive
@!multitest
let n: 2..=1000 = read()
domain Pos = 1..=2 * n + 1

fn has_triple(s: List<Pos>) -> Bool:
    print("?", s.len, s)
    (s.len - read<Int>()) % 2 == 1

let z = bsearch_first(Pos.first + 2..=Pos.last, k => has_triple((Pos.first..=k).to_list()))!
let x = bsearch_last(Pos.first..=z - 2, k => has_triple((k..=z).to_list()))!
let y = bsearch_first(x + 1..=z - 1, k => has_triple((x..=k).to_list() + [z]))!
print("!", x, y, z)
