// Kaučuk, in the forms proto/cp.py lib-run takes today (proto/cpc
// has no string interpolation, so this one runs on the interpreter). See ../coci21c2p1.cp.
// original: corpus/submissions/4143397.cpp
let n: Int = read()
var at = [0, 0, 0, 0]
for _ in 0..n:
    let kind: Str = read()
    let title: Str = read()
    let depth = 1 + (kind.len - 7) / 3
    at[depth] += 1
    for d in depth + 1..=3: at[d] = 0
    if depth == 1: print("{at[1]} {title}")
    elif depth == 2: print("{at[1]}.{at[2]} {title}")
    else: print("{at[1]}.{at[2]}.{at[3]} {title}")
