// Codeforces 2096 G (Wonderful Guessing Game): identify 1 fake coin among n
// in q parallel weighings when at most 1 weighing answer may be erased ('?').
//
// Choose q = ceil_log(3, n) + 1 so 3^(q - 1) >= n. Assign each pair (2p - 1, 2p)
// a distinct balanced-ternary code of length q - 1 in {-1, 0, +1} and append a
// checksum digit so the q digits sum to 0 (mod 3):
//   - coin 2p - 1 gets +code (+1 => Left, -1 => Right, 0 => Off),
//   - coin 2p gets -code (opposite pans),
//   - if n is odd, coin n gets the all-zero code (always Off).
// Every two coins have distinct codes with zero digit sum mod 3, so they differ
// in at least 2 rounds; after at most one '?' erasure, the unique coin matching
// every known round is the fake.

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
let q = ceil_log(3, n) + 1
domain Round = 0..q

var rounds: [Round, Side]List<Coin> = []
for pair in 1..=n / 2:
    let (a, b) = (Coin(2 * pair - 1), Coin(2 * pair))
    let sides = balanced_sides(pair, q)
    for (r, s) in Round.zip(sides):
        rounds[r, s].push(a)
        rounds[r, opposite(s)].push(b)
if n % 2 == 1:
    for r in Round: rounds[r, Side.Off].push(Coin.last)

print(q)
for r in Round:
    print(rounds[r, Side.Left].len + rounds[r, Side.Right].len, rounds[r, Side.Left], rounds[r, Side.Right])

let response: Str = read()
var hits: [Coin]Int = 0
var known = 0
for (r, ch) in Round.zip(response):
    if let side = parse_side(ch):
        known += 1
        for c in rounds[r, side]: hits[c] += 1
print(Coin.first(c => hits[c] == known)!)
