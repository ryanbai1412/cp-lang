// Source: Codeforces 2255A (Hot Potatoes at the Fairy Warehouse),
//   https://codeforces.com/contest/2255/problem/A
// Problem: 2n leprechauns sit in a circle (n <= 1e5), odd numbers Red, even Blue; s marks who
//   holds a potato. For k <= 1e9 rounds each holder keeps it or passes it clockwise if the next
//   one held none at the round's start. Each team scores the other team's holders at the end;
//   print both scores under optimal play. Multitest, sum n <= 1e5.
// Solution: the optimal result does not depend on k (the code ignores it): it is the board
//   after one round in which every free potato is passed on. A cell ends with a potato when it
//   holds one and the next cell does too (blocked), or holds none and the previous one holds
//   one. Count those cells by team, Red's score first (even leprechauns, odd 0-based indices),
//   with indices taken mod 2n; O(n).

@!multitest
let (n, _k): (1..=200_000, 1..=10 ** 9) = read()
let m = 2 * n
let s: [0..m]Char = read()

fn hit(i: Int) -> Bool:
    (s[i] == '1' and s[(i + 1) % m] == '1') or (s[i] == '0' and s[(i - 1) % m] == '1')

print((0..m).count(i => i % 2 == 1 and hit(i)), (0..m).count(i => i % 2 == 0 and hit(i)))
