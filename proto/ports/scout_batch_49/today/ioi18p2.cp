// IOI 2018 Day 1 P2, Seats (buildable today-form)
// original: corpus/submissions/4731161.cpp (grader functions give_initial_chart / swap_seats)
// stdin: h w q, the h * w seats (row, column) of contestants 0.., then q
// lines a b. stdout: the number of beautiful prefixes after each swap.
let h: 1..=1000 = read()
let w: 1..=1000 = read()
let q: 0..=50000 = read()
let people = h * w
let chart: [0..2 * people]Int = read()

var row: List<Int> = []
var col: List<Int> = []
var who: List<Int> = []  // who[r * w + c]: contestant in that seat
for _ in 0..people: who.push(0)
for p in 0..people:
    row.push(chart[2 * p])
    col.push(chart[2 * p + 1])
    who[row[p] * w + col[p]] = p

// Tree position t stands for the prefix 0..=t. bad[t] counts its top-left
// corners (a seated cell with nobody above or to the left) plus its dents
// (an empty cell with two or more seated neighbours). A prefix is a
// rectangle exactly when that total is 1, the least it can be.
var least: List<Int> = []
var times: List<Int> = []
var pending: List<Int> = []
for _ in 0..4 * people:
    least.push(0)
    times.push(1)
    pending.push(0)

fn build(o: Int, l: Int, r: Int):
    times[o] = r - l + 1
    if l == r: return
    let mid = (l + r) / 2
    build(2 * o, l, mid)
    build(2 * o + 1, mid + 1, r)

fn add(o: Int, l: Int, r: Int, ql: Int, qr: Int, d: Int):
    if ql <= l and r <= qr:
        least[o] += d
        pending[o] += d
        return
    let mid = (l + r) / 2
    if ql <= mid: add(2 * o, l, mid, ql, qr, d)
    if qr > mid: add(2 * o + 1, mid + 1, r, ql, qr, d)
    let (a, b) = (least[2 * o], least[2 * o + 1])
    least[o] = min(a, b) + pending[o]
    times[o] = (if a <= b: times[2 * o] else: 0) + (if b <= a: times[2 * o + 1] else: 0)

// arrival time of the contestant at (r, c); off the grid counts as never
fn at(r: Int, c: Int) -> Int:
    if r < 0 or r >= h or c < 0 or c >= w: return people
    who[r * w + c]

fn account(r: Int, c: Int, d: Int):
    let me = at(r, c)
    // a corner from my arrival until the first of my upper and left neighbours
    let covered = min(at(r - 1, c), at(r, c - 1))
    if me < covered: add(1, 0, people - 1, me, covered - 1, d)
    // a dent from my second neighbour's arrival until my own
    var around: List<Int> = [at(r - 1, c), at(r + 1, c), at(r, c - 1), at(r, c + 1)]
    sort(mut around)
    if around[1] < me: add(1, 0, people - 1, around[1], me - 1, d)

build(1, 0, people - 1)
for r in 0..h:
    for c in 0..w: account(r, c, 1)

for _ in 0..q:
    let a: Int = read()
    let b: Int = read()
    // the two seats and their neighbours are the only cells whose spans change
    var touched: List<Int> = []
    for p in [a, b]:
        touched.push(row[p] * w + col[p])
        if row[p] > 0: touched.push((row[p] - 1) * w + col[p])
        if row[p] + 1 < h: touched.push((row[p] + 1) * w + col[p])
        if col[p] > 0: touched.push(row[p] * w + col[p] - 1)
        if col[p] + 1 < w: touched.push(row[p] * w + col[p] + 1)
    sort(mut touched)
    var cells: List<Int> = []
    for x in touched:
        if cells.len == 0 or cells.last! != x: cells.push(x)
    for x in cells: account(x / w, x % w, -1)
    let (ra, ca) = (row[a], col[a])
    row[a] = row[b]
    col[a] = col[b]
    row[b] = ra
    col[b] = ca
    who[row[a] * w + col[a]] = a
    who[row[b] * w + col[b]] = b
    for x in cells: account(x / w, x % w, 1)
    print(times[1])
