// Source: Codeforces 2069E (A, B, AB and BA), https://codeforces.com/contest/2069/problem/E
// Problem: split a string s over {A, B} into blocks "A", "B", "AB", "BA" using at most a,
//   b, ab and ba of each; print YES or NO. |s| <= 5e5 (sum), counts <= 5e5, multitest.
// Solution: we need at least need = max(0, #A - a, #B - b) pair blocks. Cut s between equal
//   neighbours into alternating runs: an odd run of length 2k + 1 gives k pairs of any kind,
//   an even run of 2l starting with A gives l AB (or l - 1 BA), and symmetrically for B.
//   Spend the free pairs, then if one side's supply exceeds its cap, convert its largest
//   runs to the other kind one by one (each loses one pair) and check the total reaches
//   need. O(|s| log |s|) for the sort.

@!multitest
let s: Str = read()
let (a, b, ab, ba): (0..=500_000, 0..=500_000, 0..=500_000, 0..=500_000) = read()

fn can_shift(mut over: List<Int>, cap: Int, other_avail: Int, need0: Int) -> Bool:
    var rem = over.sum()
    var need = need0
    if min(rem, cap) + other_avail >= need: return true
    over.sort()
    for l in over.rev:
        rem -= l
        need -= l - 1
        if min(rem, cap) + other_avail >= need: return true
    false

fn possible() -> Bool:
    let cnt_a = s.count('A')
    let cnt_b = s.count('B')
    let base_need = max(0, cnt_a - a, cnt_b - b)
    if base_need > ab + ba: return false

    var wild = 0
    var ab_runs = List<Int>()
    var ba_runs = List<Int>()
    for r in s.runs(link: (x, y) => x != y):
        if r.len % 2 == 1:
            wild += r.len / 2
        elif r.first! == 'A':
            ab_runs.push(r.len / 2)
        else:
            ba_runs.push(r.len / 2)

    let need = base_need - wild
    let sum_ab = ab_runs.sum()
    let sum_ba = ba_runs.sum()
    if sum_ab + sum_ba < need: return false
    if sum_ab > ab: can_shift(mut ab_runs, ab, min(sum_ba, ba), need)
    else: can_shift(mut ba_runs, ba, min(sum_ab, ab), need)

print(yes(possible()))
