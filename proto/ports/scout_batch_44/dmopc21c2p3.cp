// DMOPC '21 Contest 2 P3 — interactive: recover hidden 60-bit values from
// trie distances (the parent of x is x >> 1, the root is 0).
// original: corpus/submissions/3946741.cpp
// CP Lang 0.2.2
let n: Int = read()

fn probe(guess: Int) -> Int:
    let d: Int = ask(guess, limit: 31_999)
    d

for _ in 0..n:
    let depth = probe(0)
    if depth == 0: continue
    // Every bit of `cur` from `low` upward is known to be right.
    var low = depth - 1
    var cur = 1 << low
    while true:
        let guess = cur | rand(0, (1 << low) - 1)
        let d = probe(guess)
        if d == 0: break
        // Same depth, so d is twice the height of the first wrong bit.
        low = d / 2 - 1
        cur = (cur | ((guess >> low) << low)) ^ (1 << low)

@interactor
fn judge(mut sol: Interactor):
    let n: Int = read()
    domain T = 0..n
    let hidden: [T]Int = read()
    sol.send(n)
    for i in T:
        let x = hidden[i]
        while true:
            let g: Int = sol.recv()
            var a = x
            var b = g
            var d = 0
            while a != b:
                if a > b: a = a >> 1
                else: b = b >> 1
                d += 1
            sol.send(d)
            if d == 0: break
