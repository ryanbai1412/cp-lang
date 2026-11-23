// CF 1856C To Become Max
// in: t; n k, a_1..a_n | out: per test the largest max(a) reachable with at most k operations
@!multitest
let n: 2..=1000 = read()
let k: 1..=1e8 = read()
domain I = 0..n
let a = read<[I]1..=1e8>()

fn reaches(i: I, x: Int) -> Bool:
    var need = x
    var spent = 0
    for y in a[i..]:
        if y >= need: return spent <= k
        spent += need - y
        if spent > k: return false
        need -= 1
    false

print(I.map(i => (a[i]..=a[i] + k).last_true(x => reaches(i, x))!).max()!)
