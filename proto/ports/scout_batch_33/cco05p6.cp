// CCO 2005 P6, Windows: n rectangular windows are opened in order on a
// screen, each on top of the earlier ones. Each of q clicks hits the
// topmost window containing the point: print its number (0 if none) and
// bring that window to the top.
// input: the screen size (unused), n, then `left top right bottom` per
// window, q, then `x y` per click
// original: corpus/submissions/5150823.cpp
let (_w, _h, n): (Int, Int, Int) = read()
domain Win = 1..=n
let frame: [Win](x1: Int, y2: Int, x2: Int, y1: Int) = read()
var stack: List<Win> = [w for w in Win]     // bottom to top
let q: Int = read()
for _ in 0..q:
    let (x, y): (Int, Int) = read()
    var hit: ?Int = none                    // position in the stack
    for i in 0..stack.len:
        let f = frame[stack[i]]
        if f.x1 <= x and x <= f.x2 and f.y1 <= y and y <= f.y2: hit = i
    if let i = hit:
        let w = stack[i]
        print(w)
        // WISH C3: `stack.remove_at(i)`; `rotate_left` on a slice would also do
        for j in i..stack.len - 1: stack[j] = stack[j + 1]
        stack[stack.len - 1] = w
    else: print(0)
