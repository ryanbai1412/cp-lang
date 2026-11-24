// Source: NOI 2004 P1 (The Depressed Cashier), https://dmoj.ca/problem/noi04p1
// limits: 600 ms, 64 MB
// Problem: n <= 3e5 commands and a wage floor. "I x" hires at salary x (gone at once, and
//   not counted, if below the floor); "A x" raises every salary by x; "S x" cuts every
//   salary by x, then everyone below the floor leaves; "F k" prints the k-th highest salary
//   or -1. At most 100 A/S commands. Finally print how many people left.
// Solution: everyone moves together, so keep base = salary - (raise so far) in a std
//   `SortedBag`; the bases' order never changes. A cut drains the lowest bases while they
//   fall below the floor, and "F k" is the bag's (size - k)-th smallest. O(n log n).
// original: corpus/submissions/3738379.cpp
// Bounds: q: 1..=3e5; floor: 0..=1e9; cmds.x: 0..=1e9
let (q, floor): (1..=3e5, 0..=1e9) = read()
domain Cmd = 0..q
let cmds: [Cmd](op: Char, x: 0..=1e9) = read()

var staff = SortedBag<Int>()  // base salaries
var (raise, left) = (0, 0)
for (op, x) in cmds:
    match op:
        'I' => if x >= floor: staff.add(x - raise)
        'A' => raise += x
        'S' =>
            raise -= x
            left += staff.drain(while: b => b + raise < floor).len
        _ => print(if x > staff.len: -1 else: staff.nth(staff.len - x)! + raise)
print(left)
