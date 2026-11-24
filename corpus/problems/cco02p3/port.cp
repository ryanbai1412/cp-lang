// Source: CCO 2002 Stage 2 Day 1 P3 (Return To Blind Man's Bluff), https://dmoj.ca/problem/cco02p3
// Problem: interactive. A blind robot is on an n x m board with unit obstacles; it is given n,
//   m, its start row and column and its facing (N, E, S, W). It may turn left (L), turn right
//   (R) or try a step forward (F, answered 1 if it moved, 0 if blocked by a wall or obstacle).
//   Every free cell is reachable; print the board, '.' free and 'X' obstacle, north up.
// Solution: depth-first search with the robot: from each cell try the four directions once
//   (skipping cells already known), recurse after a successful step, then turn around, step
//   back and turn so the robot is left as it was found. Unvisited cells are obstacles.
//   O(n m) moves.
// Protocol: print L, R or F (F reads 0 or 1), then A, then the map.
// original: corpus/submissions/5040084.cpp
// Bounds: n, m: 1..=80
let n: 1..=80 = read()
let m: 1..=80 = read()
domain Row = 1..=n
domain Col = 1..=m
type Cell = (Row, Col)
var at: Cell = read()
let facing: Char = read()

const DIRS = [(-1, 0), (0, 1), (1, 0), (0, -1)]  // N E S W, clockwise
fn step(p: Cell, d: (Int, Int)) -> ?Cell:        // WISH L1: p.step(d) over Dir4
    if let r = Row.at(p.0 + d.0) and let c = Col.at(p.1 + d.1):
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
