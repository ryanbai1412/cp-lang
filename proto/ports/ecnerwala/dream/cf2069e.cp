// Codeforces 2069 E: A, B, AB and BA (dream).
// Singles cost a and b, so we need `need` pairs. Pairs live inside maximal
// alternating runs: an odd run of 2k+1 gives k pairs of either kind; an even
// run of 2k gives k pairs of the kind it starts with, or only k-1 of either.
// Fill each kind's budget with its own even runs, shortest first; what does
// not fit is flexible, and flexible pairs fill whatever budget is still free.
@!multitest
let s: Str = read()
let a, b, ab, ba: 0..=5e5 = read()
let need = max(s.count('A') - a, s.count('B') - b, 0)

var room = {'A': ab, 'B': ba}    // budget by the letter a pair starts with
var flexible = 0
for run in s.chunk_by((x, y) => x != y).sort(by: r => r.len):
    let k = run.len / 2
    if run.len % 2 == 1: flexible += k
    elif k <= room[run.first]: room[run.first] -= k
    else: flexible += k - 1
let unused = max(room.values.sum() - flexible, 0)
print(if ab + ba - unused >= need: "YES" else: "NO")
