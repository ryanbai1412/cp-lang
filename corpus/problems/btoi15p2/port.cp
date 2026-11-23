// BOI 2015 P2, Editor: n operations on a text. A positive number types
// that character: an operation of level 0. A negative number -L is an undo
// of level L: it reverts the latest still-active operation of level below
// L, which may itself be an undo (and is then redone). After every
// operation print the last character of the text, 0 if it is empty.
//
// The state after operation i is a chain of active operations. Typing
// puts i on top of the previous chain. An undo of level L finds the
// topmost operation u below level L in the previous chain and continues
// with whatever was active just before u; binary lifting over "next lower
// in my chain" finds both in O(log n), because levels along a chain's
// jump pointers only get lower.
// original: corpus/submissions/5640898.cpp
let n: Int = read()
domain Op = 0..n
domain Lg = 0..19
let ops: [Op]Int = read()
fn level(i: Op) -> Int = max(0, -ops[i])

// up[j, i]: 2^j steps down the chain under operation i (undos only)
var up: [Lg, Op]?Op = None

// the topmost operation of level < x in the chain whose top is `top`
fn below(top: ?Op, x: Int) -> ?Op:
    let t = top else: return None
    if level(t) < x: return t
    var u = t
    for j in Lg.rev:
        if let v = up[j, u]:
            if level(v) >= x: u = v
    up[Lg.first!, u]

for i in Op:
    if ops[i] < 0:
        let undone = below(i.prev, level(i))!
        up[Lg.first!, i] = below(undone.prev, level(i))
        for j in Lg:
            let lower = j.prev else: continue
            let half = up[lower, i] else: break
            up[j, i] = up[lower, half]
    // WISH C2: `print(if let t = below(i, 1): ops[t] else: 0)`; `if let` is a statement only.
    let typed = below(i, 1)
    print(if typed == None: 0 else: ops[typed!])
