// DMOPC '21 Contest 2 P3 — interactive: recover hidden 60-bit values from
// trie distances (the parent of x is x >> 1, the root is 0).
// original: corpus/submissions/3946741.cpp
let n: Int = read()

// FRICTION F4: the query has no tag and the answer is implicit ("distance 0"
// means found), so `ask` carries neither a prefix nor a final `answer`.
fn probe(guess: 0..(1 << 60)) -> Int = ask(guess, limit: 31_999)

for _ in 0..n:
    let depth = probe(0)
    if depth == 0: continue
    // Every bit of `cur` from `low` upward is known to be right.
    var low = depth - 1
    var cur = 1 << low
    loop:
        let guess = cur | rand(0..(1 << low))
        let d = probe(guess)
        if d == 0: break
        // Same depth, so d is twice the height of the first wrong bit.
        low = d / 2 - 1
        cur = (cur | (guess >> low << low)) ^ (1 << low)

@interactor fn judge():
    let hidden: List<Int> = read()
    for x in hidden:
        loop:
            var (a, b, d) = (x, read_output<Int>(), 0)
            while a != b:
                if a > b: a >>= 1 else: b >>= 1
                d += 1
            print(d)
            if d == 0: break
