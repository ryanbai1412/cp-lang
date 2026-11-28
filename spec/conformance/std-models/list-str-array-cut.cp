// expect: ok, output "ok"
// input: "1 300\n"
// model: List
// model: Str
// model: Array
// model: Cut
// Against brute force: seeded random edits of a `List` and a `Str`, each checked after every
// step against a copy rebuilt cell by cell, and every `[D]T` cell and `D.cuts` cut read back.
use std.rand.{Rng}

domain D = 0..5

let seed = read<Int>()
let n = read<Int>()
var rng = Rng(seed)
var bad = ""
var xs: List<Int> = []
var m: List<Int> = []
let abc = ['a', 'b', 'c']
var s = ""
var sm: List<Char> = []
for step in 0..n:
    if bad != "": break
    let op = rng.int(0, 9)
    let x = rng.int(-9, 9)
    if op == 0:
        xs.push(x)
        m = [if k < m.len: m[k] else: x for k in 0..m.len + 1]
    elif op == 1:
        if let v = xs.pop():
            if m.len == 0 or v != m[m.len - 1]: bad = "pop {step}"
            m = [m[k] for k in 0..m.len - 1]
        elif m.len != 0: bad = "pop {step}"
    elif op == 2:
        let i = rng.int(0, m.len)
        xs.insert(i, x)
        m = [if k < i: m[k] else: if k == i: x else: m[k - 1] for k in 0..m.len + 1]
    elif op == 3 and m.len > 0:
        let i = rng.int(0, m.len - 1)
        if xs.remove(i) != m[i]: bad = "remove {step}"
        m = [if k < i: m[k] else: m[k + 1] for k in 0..m.len - 1]
    elif op == 4 and m.len > 0:
        let (i, j) = (rng.int(0, m.len - 1), rng.int(0, m.len - 1))
        xs.swap(i, j)
        m = [if k == i: m[j] else: if k == j: m[i] else: m[k] for k in 0..m.len]
    elif op == 5:
        let k = rng.int(0, m.len + 1)
        xs.truncate(k)
        m = [m[t] for t in 0..min(k, m.len)]
    elif op == 6:
        let ys = [rng.int(-9, 9) for _ in 0..rng.int(0, 3)]
        xs.extend(ys)
        m = [if k < m.len: m[k] else: ys[k - m.len] for k in 0..m.len + ys.len]
    elif op == 7:
        xs.fill(x)
        m = [x for _ in 0..m.len]
    elif op == 8:
        let k = rng.int(0, 6)
        xs.reserve(k)
        xs.resize(k, x)
        m = [if t < m.len: m[t] else: x for t in 0..k]
    else:
        let c = abc[rng.int(0, 2)]
        if x < 0: s.push(c)
        else: s = s + "{c}{c}"
        sm = [if k < sm.len: sm[k] else: c for k in 0..sm.len + (if x < 0: 1 else: 2)]
    if xs.len != m.len or xs.is_empty != (m.len == 0): bad = "len {step}"
    for k in 0..m.len:
        if xs.get(k) != m[k]: bad = "get {step}"
    if s.len != sm.len or s.is_empty != (sm.len == 0): bad = "str len {step}"
    for k in 0..sm.len:
        if s.get(k) != sm[k]: bad = "str get {step}"
let w = rng.int(-9, 9)
let a = [D] i => i.ord * w
if a.len != D.count or a.is_empty: bad = "array len"
for i in D:
    if a[i] != i.ord * w: bad = "array get"
for k in 0..=D.count:
    let c = D.cuts.nth(k)
    if c.ord != k: bad = "cut ord"
    if let l = c.l:
        if k == 0 or l != D.nth(k - 1): bad = "cut l"
    elif k != 0: bad = "cut l"
    if let r = c.r:
        if k == D.count or r != D.nth(k): bad = "cut r"
    elif k != D.count: bad = "cut r"
print(if bad == "": "ok" else: bad)
