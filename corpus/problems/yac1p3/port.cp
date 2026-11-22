// YAC 1 P3 (today/ runnable subset) — original: corpus/submissions/4380584.cpp

let n: 1..=1000 = read()
let t: 1..=1000 = read()

for _ in 0..t:
    if n % 2 == 1:
        print(1)
        print((n + 1) / 2, (n + 1) / 2)
    else:
        print(2)
    var playing = true
    while playing:
        let x: 0..=1000 = read()
        let y: 0..=1000 = read()
        if x == 0 and y == 0:
            playing = false
        else:
            print(n + 1 - x, n + 1 - y)
