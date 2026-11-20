// Runnable port (proto/cp.py lib-run) of etsr23r2p3.cp
// original: corpus/submissions/5494453.cpp

let tcs: Int = read()
for _ in 0..tcs:
    let (a, b, c): (Int, Int, Int) = read()
    let xs = sorted([a, b, c])
    let step = 1 << (ctz(xs[0]) + 1)
    if (xs[1] - xs[0]) % step == 0 and (xs[2] - xs[1]) % step == 0:
        print("YES")
    else:
        print("NO")
