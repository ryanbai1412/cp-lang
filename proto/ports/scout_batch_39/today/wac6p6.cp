// WAC 6 P6 (Hard) (today/ runnable subset) — original: corpus/submissions/5122001.cpp

let tcs: 1..=10_000 = read()

for _ in 0..tcs:
    var c0: 0..=100 = read()
    var c1: 0..=100 = read()
    var c2: 0..=100 = read()
    if c0 > c1:
        let t = c0
        c0 = c1
        c1 = t
    if c1 > c2:
        let t = c1
        c1 = c2
        c2 = t
    if c0 > c1:
        let t = c0
        c0 = c1
        c1 = t
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
        var cnt: [0..3]Int = 0
        var bin: [0..3]Int = -1
        var next_bin = 1
        for _ in 0..100:
            let ch: Char = read()
            var x = 2
            if ch == 'R':
                x = 0
            elif ch == 'G':
                x = 1
            cnt[x] += 1
            if cnt[x] > a:
                if bin[x] == -1:
                    bin[x] = next_bin
                    next_bin += 1
                print("PLACE", bin[x])
            else:
                print("DISCARD")
    else:
        var cnt: [0..3]Int = 0
        var bin: [0..3]Int = -1
        bin[0] = 1
        bin[1] = 2
        for _ in 0..100:
            let ch: Char = read()
            var x = 2
            if ch == 'R':
                x = 0
            elif ch == 'G':
                x = 1
            cnt[x] += 1
            if cnt[x] > b and bin[x] == -1:
                let tmp = bin[x]
                bin[x] = bin[0]
                bin[0] = tmp
                print("EMPTY", bin[x])
            if bin[x] == -1:
                print("DISCARD")
            else:
                print("PLACE", bin[x])
