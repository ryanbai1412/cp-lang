
let n = 100_000
let q = 100_000
let vmax = 100_000
domain I = 0..n
var seed = 123456789
fn next_rng() -> Int:
    seed = (seed * 1103515245 + 12345) % 2147483648
    seed

let a: [I]0..vmax = [I] _ => next_rng() % vmax
var queries: List<Range<I>> = []
for _ in 0..q:
    let x = next_rng() % n
    let y = next_rng() % n
    queries.push(I(min(x, y))..=I(max(x, y)))

var freq: [vmax]0..=n = 0
var distinct = 0
fn add_pos(p: I):
    unchecked:
        let v = a[p]
        if freq[v] == 0: distinct += 1
        freq[v] += 1
fn rem_pos(p: I):
    unchecked:
        let v = a[p]
        freq[v] -= 1
        if freq[v] == 0: distinct -= 1
fn ans_q(qi: Int) -> Int: distinct

let ans = mo(queries, add: add_pos, remove: rem_pos, answer: ans_q)
var acc = 0
for i in 0..q: acc += (i + 1) * ans[i]
print(acc)
