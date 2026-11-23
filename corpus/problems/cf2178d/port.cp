// Source: Codeforces 2178D, Xmas or Hysteria, https://codeforces.com/contest/2178/problem/D
// Problem: multitest, sum n <= 2e5; elves with distinct attacks a_i <= 1e9, health a_i. When
//   alive x attacks alive y, y loses a_x and x loses a_y. Print a list of attacks after which
//   exactly m elves survive, each of them having attacked, or -1.
// Solution: every survivor kills someone, so 2m > n fails. Sort by attack. For m >= 1 the
//   n - 2m weakest die in a chain, each attacking the next, then each of the m strongest kills
//   the elf m below it. For m == 0 the weak tail attacks the strongest from the top down:
//   take the largest start s whose attacks sum to at least its health. O(n log n).
@!multitest
let (n, m): (2..=200_000, 0..=200_000) = read()
domain Elf = 1..=n
let attack: [Elf]Int = read()
let ord = Elf.sort(by: e => attack[e])

fn plan() -> ?List<(Elf, Elf)>:
    if 2 * m > n: return None
    if m >= 1:
        let k = n - 2 * m
        return ord[..=k].adjacent.collect() + [
            (ord[n - 1 - i], ord[n - m - 1 - i]) for i in 0..m
        ]
    let top = ord.last!
    let pref = prefix(ord.map(e => attack[e]))
    let s = (0..n - 1).last_true(k => pref.fold(k..n - 1) >= attack[top]) ?? return None
    ord[..=s].adjacent.collect() + [(e, top) for e in ord[s..n - 1].rev]

let moves = plan() ?? return print(-1)
print(moves.len)
for (a, b) in moves: print(a, b)
