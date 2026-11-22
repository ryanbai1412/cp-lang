// Codeforces 2096 G (Wonderful Guessing Game): identify 1 fake coin among n
// in q parallel weighings when at most 1 weighing answer may be erased ('?').
//
// Choose q = ceil(log_3 n) + 1 so 3^(q - 1) >= n. Assign each pair (2p - 1, 2p)
// a distinct balanced-ternary code of length q - 1 in {-1, 0, +1} and append a
// checksum digit so the q digits sum to 0 (mod 3):
//   - coin 2p - 1 gets +code (+1 => Left, -1 => Right, 0 => Off),
//   - coin 2p gets -code (opposite pans),
//   - if n is odd, coin n gets the all-zero code (always Off).
// Every two coins have distinct codes with zero digit sum mod 3, so they differ
// in at least 2 rounds; after at most one '?' erasure, the unique coin matching
// every known round is the fake.

@!interactive

enum Side:
    Left
    Right
    Off

struct Weighing:
    left: List<Int>
    right: List<Int>
    off: List<Int>

fn ceil_log3(n: Int) -> Int:
    var k = 0
    var p = 1
    while p < n:
        k += 1
        p *= 3
    k

fn balanced_code(id: Int, len: Int) -> List<Int>:
    var v = id
    var total = 0
    var digits: List<Int> = []
    for k in 0..len:
        let r = (if k == len - 1: -total else: v) % 3
        let d = if r == 2: -1 else: r
        digits.push(d)
        total += d
        v = (v - d) / 3
    digits

fn parse_side(ch: Char) -> ?Side = match ch:
    'L' => some(Side.Left)
    'R' => some(Side.Right)
    'N' => some(Side.Off)
    _ => none

@!multitest
let n: 2..=200_000 = read()
domain Coin = 1..=n
let q = ceil_log3(n) + 1
domain Round = 0..q

var rounds: [Round]Weighing = Weighing([], [], [])
for pair in 1..=n / 2:
    let (a, b) = (2 * pair - 1, 2 * pair)
    let code = balanced_code(pair, q)
    for r in Round:
        match code[r.ord]:
            1 =>
                rounds[r].left.push(a)
                rounds[r].right.push(b)
            -1 =>
                rounds[r].left.push(b)
                rounds[r].right.push(a)
            _ =>
                rounds[r].off.push(a)
                rounds[r].off.push(b)
if n % 2 == 1:
    for r in Round: rounds[r].off.push(n)

print(q)
for r in Round:
    print(rounds[r].left.len + rounds[r].right.len, rounds[r].left, rounds[r].right)

let response: Str = read()
var hits: [Coin]Int = 0
var known = 0
for r in Round:
    if let side = parse_side(response[r.ord]):
        known += 1
        let coins = match side:
            Side.Left => rounds[r].left
            Side.Right => rounds[r].right
            Side.Off => rounds[r].off
        for c in coins: hits[Coin(c)] += 1
print(Coin.first(c => hits[c] == known)!)
