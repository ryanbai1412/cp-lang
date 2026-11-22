// Runnable port (proto/cp.py lib-run) of mccc5j3.cp
// original: corpus/submissions/5333116.cpp

let (n, k, m): (Int, Int, Int) = read()
domain I = 0..n
let a: [I]Int = read()
let b: [I]Int = read()

var pos = Map<Int, List<Int>>()
for i in I:
    let x = a[i]
    if x not in pos: pos[x] = []
    pos[x].push(i.to_idx)

fn first_ge(xs: List<Int>, target: Int) -> Int:
    let lo = xs.first_ge(target) ?? xs.len
    lo

var ans = 0
for i in I:
    let need = m - b[i]
    if need in pos:
        let xs = pos[need]
        if k == 0:
            ans += xs.len
        else:
            let idx = i.to_idx
            ans += first_ge(xs, idx - k + 1)
            ans += xs.len - first_ge(xs, idx + k)

print(ans)
