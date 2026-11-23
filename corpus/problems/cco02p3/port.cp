// CCO 2002 P3, Return to Blind Man's Bluff (interactive): a robot stands
// in a room it cannot see. It may turn left (L), turn right (R) or try to
// step forward (F, answered 1 if it moved and 0 if it hit something).
// Print A and then a map of every cell it can reach.
// original: corpus/submissions/5040084.cpp
let n: 1..=80 = read()
let m: 1..=80 = read()
domain Row = 1..=n
domain Col = 1..=m
type Cell = (Row, Col)
var at: Cell = read()
let facing: Char = read()

const DIRS = [(-1, 0), (0, 1), (1, 0), (0, -1)]  // N E S W, clockwise
fn step(p: Cell, d: (Int, Int)) -> ?Cell:        // WISH L1: p.step(d) over Dir4
    if let r = p.0 + d.0 and let c = p.1 + d.1:
        return (r, c)
    return None

var face = 0  // WISH L2: var face: Dir4 = read(); face.cw, face.ccw
if facing == 'E': face = 1
if facing == 'S': face = 2
if facing == 'W': face = 3

fn right():
    face = (face + 1) % 4
    print("R")
fn left():
    face = (face + 3) % 4
    print("L")
fn forward() -> Bool:
    let moved: 0..=1 = ask("F")
    if moved == 1: at = step(at, DIRS[face])!
    moved == 1

var open: [Row, Col]Bool = false
var wall: [Row, Col]Bool = false
// Look all four ways from here; every call leaves the robot as it found it.
fn explore():
    open[at] = true
    for side in 0..4:
        if let q = step(at, DIRS[face]) and not open[q] and not wall[q]:
            if forward():
                explore()
                right()  // WISH L14: with forward(): explore() steps back by itself
                right()
                forward()
                left()
                continue
            wall[q] = true
        right()

explore()
print("A")
print([Row, Col] (r, c) => if open[r, c]: '.' else: 'X')
