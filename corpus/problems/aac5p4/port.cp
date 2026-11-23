// Source: An Animal Contest 5 P4 (Number Game), https://dmoj.ca/problem/aac5p4
// Problem: a board 1..n (n <= 1e6) with a piece at x (1 <= x <= n); each number 1..n - 1 may
//   be used once to move the piece that far forward or backward, staying on the board. Print
//   the least number of moves M after which no move is possible, then such a sequence as
//   signed distances (+ forward, - backward), one per line.
// Solution: the piece is stuck once it rests on a middle cell t with every size up to h = n / 2
//   used (a size above h overshoots both ends from there), so M >= h. Signing 1..h to sum to
//   t - x needs x + h(h+1)/2 + t even: an even n picks t = h or h + 1 to match; an odd n has
//   one middle and, on a parity miss, first spends the smallest odd size e > h (after a
//   move of h if e would leave the board), so M = h + 1 (n = 3, x = 2 is 1, -2). The forward
//   sizes are taken greedily from h down, then the walk steps toward t with the largest size
//   left on that side, which keeps the piece on the board. O(n).
// original: corpus/submissions/4239900.cpp

let n: 1..=1e6 = read()
let x: 1..=n = read()

let h = n / 2
var moves: List<Int> = []
var p = x
var k = h
var t = h + 1
if n == 3 and x == 2:
    moves = [1, -2]
    k = 0
    t = 1
elif n % 2 == 0:
    if (x + h * (h + 1) / 2 + h) % 2 == 0: t = h
elif (x + h * (h + 1) / 2 + t) % 2 == 1:
    let e = h + 1 + h % 2
    if (p - t).abs < e - h:
        let s = if p <= t: h else: -h
        moves.push(s)
        p += s
        k = h - 1
    let s = if p <= t: e else: -e
    moves.push(s)
    p += s

var need = (k * (k + 1) / 2 + t - p) / 2
var fwd: List<Int> = []
var back: List<Int> = []
for s in (1..=k).rev:
    if s <= need:
        fwd.push(s)
        need -= s
    else:
        back.push(s)
var i = 0
var j = 0
while i < fwd.len or j < back.len:
    if j == back.len or (i < fwd.len and (p < t or (p == t and 2 * t <= n + 1))):
        moves.push(fwd[i])
        p += fwd[i]
        i += 1
    else:
        moves.push(-back[j])
        p -= back[j]
        j += 1

print(moves.len)
print(moves, sep: "\n")

@check
fn check():
    // the least count from the original's closed form; the moves are replayed on the board
    var least = n / 2
    if n % 2 == 1 and n > 1:
        let r = (n - 3) / 2 % 4
        if (r == 0 or r == 3) != (x % 2 == 1): least += 1
    if n == 1: least = 0
    let m: Int = read_output()
    assert m == least
    var used: [1..=n]Bool = false
    var at = x
    for _ in 0..m:
        let d: Int = read_output()
        assert 1 <= d.abs < n and not used[d.abs]
        used[d.abs] = true
        at += d
        assert 1 <= at <= n
    for s in 1..n:
        assert used[s] or (at + s > n and at - s < 1)
