// Source: NOI 2003 P1 (Stick Game), https://dmoj.ca/problem/noi03p1
// limits: 600 ms, 64 MB
// Problem: one line (<= 1000 chars) holds a false equation ending in '#': numbers below 1e7
//   in seven-segment matchsticks, joined by + and -, with one '='. Move exactly one stick of
//   a digit (signs stay) so it becomes true, and print the new equation with its '#' (the
//   answer is unique; new leading zeros are fine), or No.
// Solution: parse into digits, each with weight = sign * place value (the right side counts
//   negative), so the equation holds when sum(weight * digit) is 0. Try every digit reshaped
//   with the same stick count (0-6, 0-9, 2-3, 3-5, 6-9), then every ordered pair where one
//   digit loses a stick and the other gains one, checking the new total in O(1): O(D^2).
// original: corpus/submissions/3738358.cpp
const STICKS = [6, 2, 5, 5, 4, 5, 6, 3, 7, 6]
let reshape = Set([(0, 6), (0, 9), (2, 3), (3, 5), (6, 9)])  // same count, one stick moved
let one_off = Set([(0, 8), (1, 7), (3, 9), (5, 6), (5, 9), (6, 8), (8, 9)])  // one stick added or removed

fn linked(pairs: Set<(Int, Int)>, a: Int, b: Int) -> Bool = (min(a, b), max(a, b)) in pairs

// the digits one stick less (loses) or one stick more (gains) than each digit
let loses = [[x for x in 0..10 if linked(one_off, v, x) and STICKS[x] < STICKS[v]] for v in 0..10]
let gains = [[x for x in 0..10 if linked(one_off, v, x) and STICKS[x] > STICKS[v]] for v in 0..10]

struct Digit(weight: Int, value: Int, at: Int)  // weight = sign * place value

let text: Str = read()
let (eq, _) = text.split_once('#')!

var digits: List<Digit> = []
var (side, i) = (1, 0)  // side: +1 left of '=', -1 right of it
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
    for at in (start..i).rev:  // units first
        digits.push(Digit(place, eq[at].ord - '0'.ord, at))
        place *= 10
let total = sum(d.weight * d.value for d in digits)

fn gain(d: Digit, to: Int) -> Int = (to - d.value) * d.weight

let moves = [
    [(a.at, d)] for a in digits for d in 0..10 if linked(
        reshape,
        a.value,
        d,
    ) and total + gain(a, d) == 0
] + [
    [
        (a.at, x),
        (b.at, y),
    ] for a in digits for x in loses[a.value] for b in digits for y in gains[b.value] if b.at != a.at and total + gain(
        a,
        x,
    ) + gain(
        b,
        y,
    ) == 0
]
if let changes = moves.first:
    var out = eq.clone()
    for (at, d) in changes: out[at] = Char('0'.ord + d)
    print("{out}#")
else:
    print("No")
