// Codeforces 2129 C1 — Interactive RBS (Easy Version)
//
// First binary search for a known `'('` position `open_i` and `')'` position
// `close_i`: if `ask(1..=n) > 0`, some adjacent `"()"` exists and binary search
// locates it; otherwise the string is `"))...((("`, so `(n, 1)` is `"()"`.
//
// Knowing `open_i = '('` and `close_i = ')'`, querying the 6-element sequence
// `[p1, close_i, open_i, p2, open_i, p2]` uniquely identifies `(p1, p2)` by its
// count `r` of balanced substrings:
// - `('(', '(')` -> `"()(((("` => `r = 1`
// - `('(', ')')` -> `"()()()"` => `r = 6`
// - `(')', '(')` -> `"))(((("` => `r = 0`
// - `(')', ')')` -> `"))()()"` => `r = 3`
// When `n` is odd, the final pair re-queries `Pos(n)` with itself.

@!interactive

@!multitest
let n: 2..=1_000 = read()
domain Pos = 1..=n
var ans: [Pos]Char = '?'

fn ask(inds: List<Pos>) -> Int:
    print("?", inds.len, inds)
    read()

var (mi, ma) = (0, n)
while ma - mi > 1:
    let md = (mi + ma) / 2
    if ask([Pos(i) for i in mi + 1..=md + 1]) > 0: ma = md else: mi = md
let open_i = Pos(mi + 1)
let close_i = Pos((mi + 1) % n + 1)

fn check2(p1: Pos, p2: Pos):
    let r = ask([p1, close_i, open_i, p2, open_i, p2])
    ans[p1] = if r == 1 or r == 6: '(' else: ')'
    ans[p2] = if r == 0 or r == 1: '(' else: ')'

for i in 1..=n by 2: check2(Pos(i), Pos(min(i + 1, n)))
print("!", Str(ans))
