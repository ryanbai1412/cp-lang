// B. Frogs Jump
// https://codeforces.com/contest/2272/problem/B
// horizontal jumps must cancel across every column edge, so they come in pairs of neighbours; vertical
// jumps need as many down as up. Doubled columns swap vertically; the row surplus d is fixed by pairing
// adjacent frogs in the richer row (2 each) or removing frogs (1 each).
@!multitest
let n: 1..=2e5 = read()
let s: Str = read()
let t: Str = read()
let top = s.count(c => c == 'f')
let bot = t.count(c => c == 'f')
let row = if top >= bot: s else: t
var pairs = 0
var run = 0
for c in row:
    if c == 'f':
        run += 1
    else:
        pairs += run / 2
        run = 0
pairs += run / 2
let d = abs(top - bot)
print(d - 2 * min(pairs, d / 2))
