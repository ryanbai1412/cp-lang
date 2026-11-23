// aac5p4: a piece at x on cells 1..=n may move each distance 1..n - 1 at most once, either way,
// staying on the board. Print the fewest moves after which no move fits, then the signed moves.
// The piece is stuck on a middle cell t once every size up to h = n / 2 is used, so the answer
// is h: sign 1..=h to sum to t - x, which needs x + h(h + 1) / 2 + t even. An even board picks
// t = h or h + 1 to fit; an odd board's parity miss first spends the smallest odd size above h.
let n = read<1..=1e6>()
let x = read<1..=n>()
let h = n / 2

var moves: List<Int> = []
var at = x
var k = h
var t = h + 1
if n % 2 == 0:
    if (x + h * (h + 1) / 2 + h) % 2 == 0: t = h
elif (x + h * (h + 1) / 2 + t) % 2 == 1:
    let e = h + 1 + h % 2
    if e == n:
        // n = 3, x = 2: no size above h fits, so stop on the end cell 1 instead
        moves = [1, -2]
        k = 0
        t = 1
    else:
        if (at - t).abs < e - h:
            moves.push(if at <= t: h else: -h)
            k = h - 1
        moves.push(if at + moves.sum() <= t: e else: -e)
    at += moves.sum()

// forward sizes greedily from k down until they cover the surplus, the rest backward
var need = (k * (k + 1) / 2 + t - at) / 2
var fwd: List<Int> = []
var back: List<Int> = []
for s in (1..=k).rev:
    if s <= need:
        fwd.push(s)
        need -= s
    else:
        back.push(s)

// step toward t with the largest size left on that side, which keeps the piece on the board
var i = 0
var j = 0
while i < fwd.len or j < back.len:
    let ahead = at < t or (at == t and 2 * t <= n + 1)
    if j == back.len or (i < fwd.len and ahead):
        moves.push(fwd[i])
        at += fwd[i]
        i += 1
    else:
        moves.push(-back[j])
        at -= back[j]
        j += 1

print(moves.len)
print(moves, sep: "\n")

@check
fn valid():
    // the fewest moves from the accepted solution's closed form; the moves replay on the board
    var least = if n == 1: 0 else: n / 2
    if n % 2 == 1 and n > 1:
        let r = (n - 3) / 2 % 4
        if (r == 0 or r == 3) != (x % 2 == 1): least += 1
    let m: Int = read_output()
    assert m == least
    var used: [1..=n]Bool = false
    var pos = x
    for _ in 0..m:
        let d: Int = read_output()
        assert 1 <= d.abs < n and not used[d.abs]
        used[d.abs] = true
        pos += d
        assert 1 <= pos <= n
    for s in 1..n:
        assert used[s] or (pos + s > n and pos - s < 1)
