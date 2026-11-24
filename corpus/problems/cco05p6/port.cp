// Source: CCO 2005 Day 2 P3 (Windows), https://dmoj.ca/problem/cco05p6
// Problem: n <= 5e4 rectangular windows (borders included) on a C x R screen (<= 1e4), each
//   later one on top of the earlier ones. Each of m <= 5e4 clicks at (x, y) hits the topmost
//   window containing the point: print its number (0 if none) and bring it to the top.
// Solution: keep the windows in a list from bottom to top. A click finds the last window containing the
//   point with rfind, removes it and pushes it back on top: O(n) per click, O(n m) in all.
// original: corpus/submissions/5150823.cpp
// Input: the screen size (unused), n, then `left top right bottom` per window, m, then `x y`
//   per click.
// Bounds: n: 1..=5e4; m: 0..=5e4
let _screen: (Int, Int) = read()
let n: 1..=5e4 = read()
domain Win = 1..=n
let frame: [Win](left: Int, top: Int, right: Int, bottom: Int) = read()
let m: 0..=5e4 = read()
let clicks: [m](x: Int, y: Int) = read()

fn hit(w: Win, x: Int, y: Int) -> Bool:
    let f = frame[w]
    f.left <= x and x <= f.right and f.bottom <= y and y <= f.top

var stack = Win.collect()  // bottom to top
for c in clicks:
    if let i = stack.keys.rfind(i => hit(stack[i], c.x, c.y)):
        let w = stack.remove(i)
        stack.push(w)
        print(w)
    else: print(0)
