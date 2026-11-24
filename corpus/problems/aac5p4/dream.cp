// AAC 5 P4: the piece is stuck on a middle cell t once every size up to h = n / 2 is used, so
// the fewest moves is h, plus one odd size above h first when an odd board misses the parity
// x + h(h + 1) / 2 + t even. Sign 1..=k to land on t: the largest sizes forward until they cover
// the surplus, the rest backward, and always step toward t so the piece stays on the board.
let n: 1..=1e6 = read()
let x: 1..=n = read()
let h = n / 2

var moves: List<Int> = []
var k = h
var t = h + 1
if n % 2 == 0:
    if (x + h * (h + 1) / 2 + h) % 2 == 0: t = h
elif (x + h * (h + 1) / 2 + t) % 2 == 1:
    let e = h + 1 + h % 2
    if e == n:
        // n = 3, x = 2: nothing above h fits, so stop on the end cell 1 instead
        moves = [1, -2]
        (k, t) = (0, 1)
    else:
        if (x - t).abs < e - h:
            moves.push(if x <= t: h else: -h)
            k = h - 1
        moves.push(if x + moves.sum() <= t: e else: -e)
var at = x + moves.sum()

var need = (k * (k + 1) / 2 + t - at) / 2
var fwd: List<Int> = []
var back: List<Int> = []
for s in (1..=k).rev:
    if s <= need:
        fwd.push(s)
        need -= s
    else:
        back.push(s)

var (i, j) = (0, 0)
while i < fwd.len or j < back.len:
    let ahead = at < t or (at == t and 2 * t <= n + 1)
    let d = if j == back.len or (i < fwd.len and ahead):
        i += 1
        fwd[i - 1]
    else:
        j += 1
        -back[j - 1]
    moves.push(d)
    at += d

print(moves.len)
print(moves, sep: "\n")
