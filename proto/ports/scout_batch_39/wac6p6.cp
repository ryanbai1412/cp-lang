// WAC 6 P6 (Hard) — original: corpus/submissions/5122001.cpp
// Idiomatic CP Lang v2 (locked spec: docs/one-page-cheat-sheet.md)

@tags('R', 'G', 'B')
enum Color = R | G | B

@multitest
var counts: [0..3]Int = read()
sort(mut counts)
let (a, b, c) = (counts[0], counts[1], counts[2])

if a + b >= 46:
    for _ in 0..100:
        let x: Color = read()
        match x:
            R => print("PLACE 1")
            G => print("PLACE 2")
            B => print("DISCARD")
elif b + c - 2 * a >= 46:
    var cnt: [Color]Int = 0
    var bin: [Color]?Int = none
    var next_bin = 1
    for _ in 0..100:
        let x: Color = read()
        cnt[x] += 1
        if cnt[x] > a:
            if bin[x] == none:
                bin[x] = next_bin
                next_bin += 1
            print("PLACE {bin[x]!}")
        else:
            print("DISCARD")
else:
    var cnt: [Color]Int = 0
    // FRICTION 39.6: no per-variant literal for small enum arrays [Color]?Int
    var bin: [Color]?Int = [Color] col => match col:
        R => 1
        G => 2
        B => none
    for _ in 0..100:
        let x: Color = read()
        cnt[x] += 1
        if cnt[x] > b and bin[x] == none:
            swap(mut bin[x], mut bin[R])
            print("EMPTY {bin[x]!}")
        if let id = bin[x]:
            print("PLACE {id}")
        else:
            print("DISCARD")
