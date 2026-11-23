// Synthesis #3 Leap 2 (Rule N1 + Invariant 4): strictly local syntactic optional narrowing.
fn bump(x: ?Int) -> Int:
    if x == None: return -1
    // diverging guard narrows `x` for the rest of the function
    x + 10

fn step(v: Int) -> ?Int:
    if v <= 1: return None
    v - 1

let n: 1..=10 = read()
let a: ?Int = if n > 2: n else: None
let b: ?Int = None

// if x != none
var r1 = 0
if a != None:
    r1 = a + 5
    // Invariant 4: redundant `!` inside a guarded block stays valid (lint only)
    r1 += a!

// if x == none ... else:
var r2 = 0
if a == None:
    r2 = -1
else:
    r2 = a * 2

// short-circuit `and` / `or`
let ok1 = a != None and a > 2
let ok2 = b == None or b > 100

// conditional expression narrowing
let r3 = if a != None: a + 1 else: 0

// while cur != none:
var cur: ?Int = n
var steps = 0
while cur != None:
    steps += cur
    cur = step(cur)

print(bump(a), bump(b), r1, r2, Int(ok1), Int(ok2), r3, steps)
