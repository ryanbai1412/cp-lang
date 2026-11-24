// Source: BOI 2015 P2 (Editor), https://dmoj.ca/problem/btoi15p2
// limits: 1500 ms, 512 MB
// Problem: n <= 3e5 operations a_i (|a_i| <= n, a_i != 0): a_i > 0 is an edit setting the
//   state to a_i (level 0), -L an undo of level L that undoes the latest active operation of
//   level < L (re-activating what that one undid, recursively). After each operation print
//   the state set by the last active edit, 0 if none.
// Solution: the active operations after step i form a chain. An edit puts i on top of chain
//   i - 1; an undo of level L finds the topmost u of level < L in chain i - 1 and continues
//   with the chain from just before u. Binary lifting over "next lower operation in my chain"
//   finds it in O(log n), as levels only fall along the jumps; the answer is the topmost
//   edit, below(i, 1). O(n log n).
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
