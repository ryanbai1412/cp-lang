// WAC 6 P6 (Hard) (today/ runnable subset) — original: corpus/submissions/5122001.cpp

let tcs: 1..=10_000 = read()
domain Col = 0..3

for _ in 0..tcs:
    var c0: 0..=100 = read()
    var c1: 0..=100 = read()
    var c2: 0..=100 = read()
    if c0 > c1:
        swap(mut c0, mut c1)
    if c1 > c2:
        swap(mut c1, mut c2)
    if c0 > c1:
        swap(mut c0, mut c1)
    let a = c0
    let b = c1
    let c = c2

    if a + b >= 46:
        for _ in 0..100:
            let ch: Char = read()
            if ch == 'R':
                print("PLACE", 1)
            elif ch == 'G':
                print("PLACE", 2)
            else:
                print("DISCARD")
    elif b + c - 2 * a >= 46:
        var cnt: [Col]Int = 0
        var bin: [Col]Int = -1
        var next_bin = 1
        for _ in 0..100:
            let ch: Char = read()
            var x = 2
            if ch == 'R':
                x = 0
            elif ch == 'G':
                x = 1
            let cx = Col(x)
            cnt[cx] += 1
            if cnt[cx] > a:
                if bin[cx] == -1:
                    bin[cx] = next_bin
                    next_bin += 1
                print("PLACE", bin[cx])
            else:
                print("DISCARD")
    else:
        var cnt: [Col]Int = 0
        var bin: [Col]Int = -1
        bin[Col(0)] = 1
        bin[Col(1)] = 2
        for _ in 0..100:
            let ch: Char = read()
            var x = 2
            if ch == 'R':
                x = 0
            elif ch == 'G':
                x = 1
            let cx = Col(x)
            cnt[cx] += 1
            if cnt[cx] > b and bin[cx] == -1:
                bin.swap(cx, Col(0))
                print("EMPTY", bin[cx])
            if bin[cx] == -1:
                print("DISCARD")
            else:
                print("PLACE", bin[cx])
