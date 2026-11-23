// Codeforces 1472D Even-Odd Game
// in: t; per test n, then a_1..a_n. out: per test "Alice", "Bob" or "Tie" under optimal play.
@!multitest
let n: 1..=2e5 = read()
let a = read<[n]1..=1e9>()
var lead = 0
for (turn, x) in a.sorted(by: desc).items:
    if turn % 2 == 0 and x % 2 == 0: lead += x
    elif turn % 2 == 1 and x % 2 == 1: lead -= x
print(if lead > 0: "Alice" elif lead < 0: "Bob" else: "Tie")
