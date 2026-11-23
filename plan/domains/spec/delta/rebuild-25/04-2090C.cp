// Codeforces 2090C Dining Hall: https://codeforces.com/problemset/problem/2090/C
// A table cell (x, y) is x + y steps away, plus 2 for the far corner (x % 3 == 2 and y % 3 == 2).
// Sort every cell of the tables with a, b <= K once by (distance, x, y): every cell closer than
// 3K + 2 is in the list, and there are more than 5e4 of them, so no guest walks past its end.
// Type-1 guests take the next free cell, type-0 guests the near corner of the next empty table.
const K = 320
fn dist(x: Int, y: Int) -> Int = x + y + (if x % 3 == 2 and y % 3 == 2: 2 else: 0)
var seats = [(d: dist(x, y), x:, y:, tab: a * (K + 1) + b)
             for a in 0..=K, b in 0..=K, x in [3 * a + 1, 3 * a + 2], y in [3 * b + 1, 3 * b + 2]]
seats.sort(by: s => (s.d, s.x, s.y))
let corners = [i for i in seats.keys if seats[i].x % 3 == 1 and seats[i].y % 3 == 1]
var seat_case: [seats.len]Int = -1
var table_case: [(K + 1) * (K + 1)]Int = -1
var case_no = 0

@!multitest
case_no += 1
let n: 1..=5e4 = read()
let kinds = read<[n]0..=1>()
var next_seat = 0
var next_table = 0
fn sit(i: Int):
    seat_case[i] = case_no
    table_case[seats[i].tab] = case_no
    print(seats[i].x, seats[i].y)
for kind in kinds:
    if kind == 1:
        while seat_case[next_seat] == case_no: next_seat += 1
        sit(next_seat)
    else:
        while table_case[seats[corners[next_table]].tab] == case_no: next_table += 1
        sit(corners[next_table])
