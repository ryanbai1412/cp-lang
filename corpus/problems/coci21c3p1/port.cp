// Lista, in the forms proto/cpc builds today. See ../coci21c3p1.cp.
// original: corpus/submissions/4196130.cpp
let n: Int = read()
let k: Int = read()
let a: [n]Int = read()

fn repeats(l: Int, len: Int) -> Bool:
    for i in l..l + (k - 1) * len:
        if a[i] != a[i + len]: return false
    true

fn find() -> ?(Int, Int):
    for l in 0..n:
        for len in 1..=(n - l) / k:
            if repeats(l, len): return (l, len)
    None

if let hit = find():
    print(hit.1)
    print([a[i] for i in hit.0..hit.0 + hit.1])
else: print(-1)
