// 1749C Number Game
// in: t; n; a_1..a_n. out: per test, the largest k for which Alice wins.
@!multitest
let n = read<1..=100>()
let a = read<[n]1..=100>().sorted()

fn alice_wins(k: Int) -> Bool:
    var left = a.clone()
    for stage in 1..=k:
        let limit = k - stage + 1
        let pick = left.keys.last_true(i => left[i] <= limit) else: return false
        left.remove(pick)
        if not left.is_empty: left.remove(0)
    true

print((0..=n).last_true(k => alice_wins(k))!)
