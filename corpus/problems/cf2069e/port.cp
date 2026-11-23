// Codeforces 2069 E: partition a string over {'A', 'B'} into at most `a` A's,
// `b` B's, `ab` "AB"'s, and `ba` "BA"'s.
//
// Every pair ("AB" or "BA") uses one A and one B, so we need at least
// `need = max(0, cnt_a - a, cnt_b - b)` pairs in total. Adjacent equal letters
// cannot belong to the same pair, so splitting `s` at adjacent equal letters
// yields maximal alternating runs:
// - An odd run of length 2k + 1 can form k pairs of any mix of AB and BA
//   ("wild"), by leaving one endpoint unpaired.
// - An even run of length 2l starting with 'A' (resp. 'B') forms l AB (resp.
//   BA) pairs natively, or l - 1 pairs of the opposite type by sacrificing its
//   two endpoints.
// After spending all wild pairs, if one side's native supply exceeds its cap
// (`ab` or `ba`), convert its largest runs to the other side: dropping a run of
// `l` native pairs yields `l - 1` opposite-side pairs (reducing `need` by
// `l - 1`) while freeing `l` from the overfull side.

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

print(if possible(): "YES" else: "NO")
