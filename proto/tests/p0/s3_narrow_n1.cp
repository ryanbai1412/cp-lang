// Synthesis #3 Leap 2 (Rule N1 + Invariant 4): strictly local syntactic optional narrowing.
fn bump(x: ?Int) -> Int:
    if x == none: return -1
    // diverging guard narrows `x` for the rest of the function
    x + 10

fn step(v: Int) -> ?Int:
    if v <= 1: return none
    v - 1

let n: 1..=10 = read()
let a: ?Int = if n > 2: n else: none
let b: ?Int = none

// if x != none
var r1 = 0
if a != none:
    r1 = a + 5
    // Invariant 4: redundant `!` inside a guarded block stays valid (lint only)
    r1 += a!

// if x == none ... else:
var r2 = 0
if a == none:
    r2 = -1
else:
    r2 = a * 2

// short-circuit `and` / `or`
let ok1 = a != none and a > 2
let ok2 = b == none or b > 100

// conditional expression narrowing
let r3 = if a != none: a + 1 else: 0

// while cur != none:
var cur: ?Int = n
var steps = 0
while cur != none:
    steps += cur
    cur = step(cur)

print(bump(a), bump(b), r1, r2, Int(ok1), Int(ok2), r3, steps)
