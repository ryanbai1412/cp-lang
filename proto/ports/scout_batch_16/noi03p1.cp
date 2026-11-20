// NOI 2003 Matchstick Equation: an equation such as 1+1=3# is laid out in
// seven-segment matchstick digits. Move exactly one match, between digits
// only (signs and '=' stay), so that it becomes true, and print it; or No.
//
// Parsing is the whole problem. `terms` turns the text into signed numbers
// (everything right of '=' counts negative, so the equation holds when the
// total is 0), and each digit remembers its place value and its position in
// the text. A move is then either one digit reshaped, or one digit losing a
// match and another gaining it.
// original: corpus/submissions/3738358.cpp
const STICKS = [6, 2, 5, 5, 4, 5, 6, 3, 7, 6]
const RESHAPE = [(0, 6), (0, 9), (2, 3), (3, 5), (6, 9)]                    // same count, one match moved
const ONE_OFF = [(0, 8), (1, 7), (3, 9), (5, 6), (5, 9), (6, 8), (8, 9)]    // one match added or removed

fn linked(pairs: List<(Int, Int)>, a: Int, b: Int) -> Bool = pairs.contains((min(a, b), max(a, b)))

struct Digit(weight: Int, value: Int, at: Int)      // weight = sign * place value

let text: Str = read()
var eq = Str()
for c in text:
    if c == '#': break
    eq.push(c)

var digits: List<Digit> = []
var total = 0
var (side, i) = (1, 0)                              // side: +1 left of '=', -1 right of it
while i < eq.len:
    var sign = side
    if eq[i] == '=':
        (side, sign, i) = (-1, -1, i + 1)
    if eq[i] == '+' or eq[i] == '-':
        if eq[i] == '-': sign = -sign
        i += 1
    let start = i
    while i < eq.len and eq[i].is_digit: i += 1
    var place = sign
    for at in (start..i).rev:                       // units first
        let d = eq[at].ord - '0'.ord
        digits.push(Digit(place, d, at))
        total += place * d
        place *= 10

fn show(changes: List<(Int, Int)>):
    var out = eq.copy()
    for (at, d) in changes: out[at] = Char('0'.ord + d)
    print("{out}#")

fn solve():
    for a in digits:
        for d in 0..10:
            if linked(RESHAPE, a.value, d) and total + (d - a.value) * a.weight == 0:
                show([(a.at, d)])
                return
    for a in digits:
        for b in digits:
            if a.at == b.at: continue
            for x in 0..10:
                for y in 0..10:
                    let moved = linked(ONE_OFF, a.value, x) and linked(ONE_OFF, b.value, y)
                    let kept = STICKS[x] + STICKS[y] == STICKS[a.value] + STICKS[b.value]
                    if moved and kept and total + (x - a.value) * a.weight + (y - b.value) * b.weight == 0:
                        show([(a.at, x), (b.at, y)])
                        return
    print("No")

solve()
