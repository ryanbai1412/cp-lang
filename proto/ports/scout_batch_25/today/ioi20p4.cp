// IOI '20 P4, Packing Biscuits (buildable today-form)
// original: corpus/submissions/4733047.cpp (grader function count_tastiness)
// stdin here: q, then per call `k x` and a[0..k).
let q: 1..=1000 = read()

// `1 << (i + 1)` with a loop index is emitted as a 32-bit C++ shift today
// (silently wrong from bit 31 up); a parameter goes through the checked shift.
fn bit(i: Int) -> Int: 1 << i

fn top_bit(v: Int) -> Int:
    var i = 0
    while (v >> (i + 1)) > 0:
        i += 1
    i

fn count_tastiness(x: Int, a: List<Int>) -> Int:
    // s[i] = total tastiness of types 0..=i
    var s: List<Int> = []
    var run = 0
    for i in 0..60:
        if i < a.len:
            run += a[i] << i
        s.push(run)
    // pow[i] = number of valid y below 2^i; the tail below 2^i is one chain
    var pow: List<Int> = [1]
    for i in 0..60:
        var cap = min(bit(i + 1), s[i] / x + 1) - bit(i)
        var got = pow[i]
        while cap > 0:
            if cap == 1:
                got += 1
                cap = 0
            else:
                let j = top_bit(cap - 1)
                got += pow[j]
                cap = min(cap, s[j] / x + 1) - bit(j)
        pow.push(got)
    pow[60]

for t in 0..q:
    let k: 1..=60 = read()
    let x: 1..=1000000000000000000 = read()
    var a: List<Int> = []
    for i in 0..k:
        let v: Int = read()
        a.push(v)
    print(count_tastiness(x, a))
