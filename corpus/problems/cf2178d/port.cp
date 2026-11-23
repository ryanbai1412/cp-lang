// Codeforces 2178 D: Xmas or Hysteria.
// Each survivor must have killed a distinct elf, so 2m > n is impossible.
// With elves sorted weakest first in `ord`:
//   - m >= 1: the k = n - 2m weakest die in a chain, each attacking the next,
//     ending at ord[k]; then, from the top down, each of the m strongest
//     kills the elf m below it.
//     Every attacker outlives its weaker target, so exactly the top m survive.
//   - m == 0: the strongest must die too. Chain ord[..=s] as before, then the
//     elves ord[s..n - 1] attack the strongest from the top down; this works
//     iff their attacks sum to at least its value, so take the largest such s.
// The answer is the list of attacks, or -1 when no plan exists.
@!multitest
let (n, m): (2..=200_000, 0..=200_000) = read()
domain Elf = 1..=n
let attack: [Elf]Int = read()
let ord = Elf.sort(by: e => attack[e])

fn plan() -> ?List<(Elf, Elf)>:
    if 2 * m > n: return None
    if m >= 1:
        let k = n - 2 * m
        return [p for p in ord[..=k].adjacent] + [
            (ord[n - 1 - i], ord[n - m - 1 - i]) for i in 0..m
        ]
    let top = ord.last!
    let pref = prefix(ord.map(e => attack[e]))
    let s = (0..n - 1).last_true(k => pref.fold(k..n - 1) >= attack[top]) ?? return None
    [p for p in ord[..=s].adjacent] + [(e, top) for e in ord[s..n - 1].rev]

let moves = plan() ?? return print(-1)
print(moves.len)
for (a, b) in moves: print(a, b)
