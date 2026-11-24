// Source: Codeforces 2096G (Wonderful Guessing Game),
//   https://codeforces.com/contest/2096/problem/G
// Problem: interactive, multitest, n <= 2e5 (sum n <= 2e5). Find a hidden x in 1..=n with the
//   fewest possible q queries, all printed up front: each is an even-length list, answered L
//   (x in first half), R (second half) or N (absent), and exactly one answer is erased to '?'.
// Solution: q is the least with 3^(q - 1) >= n. Pair p = (2p - 1, 2p) gets p in balanced ternary
//   (q - 1 digits) plus a check digit making the digit sum 0 mod 3; 2p - 1 takes the code
//   (+1 Left, -1 Right, 0 Off), 2p its negation, and an odd n the zero code. Codes differ in >= 2
//   digits, so the one number matching every known answer is x. O(n q).

@!interactive

enum Side: Left | Right | Off

fn digit_to_side(d: Int) -> Side = match d:
    1 => Side.Left
    -1 => Side.Right
    _ => Side.Off

fn opposite(s: Side) -> Side = match s:
    Side.Left => Side.Right
    Side.Right => Side.Left
    Side.Off => Side.Off

fn parse_side(ch: Char) -> ?Side = match ch:
    'L' => Some(Side.Left)
    'R' => Some(Side.Right)
    'N' => Some(Side.Off)
    _ => None

fn balanced_sides(id: Int, len: Int) -> List<Side>:
    var v = id
    var total = 0
    var out: List<Side> = []
    for k in 0..len:
        let r = (if k == len - 1: -total else: v) % 3
        let d = if r == 2: -1 else: r
        out.push(digit_to_side(d))
        total += d
        v = (v - d) / 3
    out

@!multitest
let n: 2..=200_000 = read()
domain Coin = 1..=n
let q = (1..).find(q => 3 ** (q - 1) >= n)!
domain Round = 0..q

var rounds: [Round, Side]List<Coin> = []
for pair in 1..=n / 2:
    let (a, b) = (Coin(2 * pair - 1), Coin(2 * pair))
    let sides = balanced_sides(pair, q)
    for r in Round:
        let s = sides[r.ord]
        rounds[r, s].push(a)
        rounds[r, opposite(s)].push(b)
if n % 2 == 1:
    for r in Round: rounds[r, Side.Off].push(Coin.last!)

print(q)
for r in Round:
    print(
        rounds[r, Side.Left].len + rounds[r, Side.Right].len,
        rounds[r, Side.Left],
        rounds[r, Side.Right],
    )

let response: Str = read()
var hits: [Coin]Int = 0
var known = 0
for r in Round:
    if let side = parse_side(response[r.ord]):
        known += 1
        for c in rounds[r, side]: hits[c] += 1
print(Coin.find(c => hits[c] == known)!)
