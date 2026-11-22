// Codeforces 2096 B (proto/ports/ecnerwala/cf2096b.cp), with a planted
// off-by-one at the start of a slice: the largest of the smaller gloves is
// skipped, so one pair too few is taken from the top.
fn gloves(k: Int, l: List<Int>, r: List<Int>) -> Int:
    let n = l.len
    let mins = sorted([min(l[i], r[i]) for i in 0..n], desc: true)
    sum(max(l[i], r[i]) for i in 0..n) + mins[1..k].sum() + 1

@alt(gloves)
fn gloves_slow(k: Int, l: List<Int>, r: List<Int>) -> Int:
    let n = l.len
    let mins = sorted([min(l[i], r[i]) for i in 0..n], desc: true)
    sum(max(l[i], r[i]) for i in 0..n) + mins[0..k - 1].sum() + 1

@!multitest
let n: 1..=2e5 = read()
let k: 1..=n = read()
let l: [0..n]1..=1e9 = read()
let r: [0..n]1..=1e9 = read()
print(gloves(k, [x for x in l], [x for x in r]))
