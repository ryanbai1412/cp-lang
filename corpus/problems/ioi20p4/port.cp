// Source: IOI 2020 P4 (Packing Biscuits), https://dmoj.ca/problem/ioi20p4
// Problem: k <= 60 biscuit types, a[i] of type i, each of tastiness 2^i, total tastiness <= 1e18.
//   Count the y for which x <= 1e18 bags can each be packed with total tastiness exactly y.
//   q <= 1000 independent calls.
// Solution: y works iff x * (y mod 2^(i+1)) <= s[i] for every i, s[i] the tastiness of types
//   0..=i (leftovers carry upwards). pow[i] counts the working y < 2^i. For pow[i + 1], the y
//   with bit i set need y <= s[i] / x; the working lower parts under that cap are counted by
//   walking the cap's high bits j, adding pow[j] and tightening the cap by s[j] / x each time.
//   O(k^2) per call.
// original: corpus/submissions/4733047.cpp (grader function count_tastiness)
// stdin here: q, then per call `k x` and a[0..k).
// Bounds: q: 1..=1000; k: 1..=60; x: 1..=1_000_000_000_000_000_000; a: 0..=1_000_000_000_000_000_000
let q: 1..=1000 = read()
domain Bit = 0..60

for _ in 0..q:
    let (k, x): (1..=60, 1..=1_000_000_000_000_000_000) = read()
    let a: [k]0..=1_000_000_000_000_000_000 = read()
    // s.cut(i + 1): the tastiness of types 0..=i, all of which can carry up into bit i
    let s = ([Bit] i => if Int(i) < k: a[Int(i)] << Int(i) else: 0).scan(+)
    fn cap_below(i: Int) -> Int = s.cut(i + 1) / x + 1

    // below[i]: the working y < 2^i. A y with top bit i works when y < cap_below(i) and its
    // low part does; the low parts under that cap are counted by walking the cap's high bits
    var below: List<Int> = [1]
    for i in 0..60:
        var cap = min(1 << (i + 1), cap_below(i)) - (1 << i)
        var got = below[i]
        while cap > 0:
            if cap == 1:
                got += 1
                break
            let j = bit_len(cap - 1) - 1
            got += below[j]
            cap = min(cap, cap_below(j)) - (1 << j)
        below.push(got)
    print(below.last!)
