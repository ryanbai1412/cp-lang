// Runnable CP Lang v2.2 port of ioi19p5 (IOI '19 P5 - Vision Program)
// Original C++: corpus/submissions/5664246.cpp
// Runs directly under the Rust `cpl run` and `cpl build` (c521b2c).
// Emits the exact Boolean circuit instructions (NOT/AND/OR/XOR) to stdout
// so the differential harness evaluates the circuit on all pairs of cells and
// checks exact equivalence against 5664246.cpp.
// Workarounds vs blind first draft (ioi19p5_draft.cp):
// - No bodyless extern `@grader fn` in Rust syntax -> explicit gate emitter
// - No `(mp[k] ??= []).push(x)` expression syntax -> check `mp.contains(k)` then update
// - No nested `@memo` closures -> `?Int` lazy cache using v2.2 `??=`!

var next_id = 0
var gate_ops: List<Int> = []
var gate_args: List<List<Int>> = []

// op codes: 1=NOT, 2=AND, 3=OR, 4=XOR
fn add_not(n: Int) -> Int:
    let id = next_id
    next_id += 1
    gate_ops.push(1)
    gate_args.push([n])
    id

fn add_and(ns: List<Int>) -> Int:
    let id = next_id
    next_id += 1
    gate_ops.push(2)
    gate_args.push(ns)
    id

fn add_or(ns: List<Int>) -> Int:
    let id = next_id
    next_id += 1
    gate_ops.push(3)
    gate_args.push(ns)
    id

let rows: Int = read()
let cols: Int = read()
let kv: Int = read()
next_id = rows * cols

var b_true: ?Int = none
var b_false: ?Int = none

fn g_true() -> Int:
    if b_true == none:
        var cq: List<Int> = []
        for r in 0..rows:
            for c in 0..cols:
                cq.push(r * cols + c)
        b_true = add_or(cq)
    b_true ?? 0

fn g_false() -> Int:
    if b_false == none:
        b_false = add_not(g_true())
    b_false ?? 0

fn go_cell(kind: Int) -> List<Int>:
    var mp: Map<Int, List<Int>> = []
    for r in 0..rows:
        for c in 0..cols:
            let key = if kind == 0: r + c else: r - c
            if not mp.contains(key):
                var empty: List<Int> = []
                mp[key] = empty
            mp[key].push(r * cols + c)
    var ans: List<Int> = []
    for (_, vs) in mp:
        ans.push(add_or(vs))
    ans

fn has_k_diff(arr: List<Int>, d: Int) -> Int:
    let n = arr.len
    if d >= n:
        return g_false()
    var cq: List<Int> = []
    for i in 0..(n - d):
        cq.push(add_and([arr[i], arr[i + d]]))
    add_or(cq)

fn has_ge_k_diff(arr: List<Int>, d: Int) -> Int:
    let n = arr.len
    if d >= n:
        return g_false()
    var cq: List<Int> = []
    var suff = arr[0]
    for i in d..n:
        suff = add_or([suff, arr[i - d]])
        cq.push(add_and([arr[i], suff]))
    add_or(cq)

let c1 = go_cell(0)
let c2 = go_cell(1)
let has_k = add_or([has_k_diff(c1, kv), has_k_diff(c2, kv)])
let has_gt_k = add_or([has_ge_k_diff(c1, kv + 1), has_ge_k_diff(c2, kv + 1)])
let _ = add_and([has_k, add_not(has_gt_k)])

print(gate_ops.len)
for i in 0..gate_ops.len:
    var line: List<Int> = [gate_ops[i], gate_args[i].len]
    for a in gate_args[i]:
        line.push(a)
    print(line)
