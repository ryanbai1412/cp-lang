// EGOI '22 P1 (SubsetMex): fewest insertions so that repeatedly replacing a
// subset by its mex can produce n. Runnable on proto/cpc today.
// original: corpus/submissions/5198974.cpp
let t: Int = read()
for _ in 0..t:
    let n: 1..=50 = read()
    let have: [0..n]Int = read()
    // need[i]: copies of i the plan consumes; a shortfall at i is bought once
    // and costs one more copy of every smaller value
    var need: [0..=n]Int = 0
    need[n] = 1
    var ans = 0
    for i in (0..=n).rev:
        let short = need[i] - (if i < n: have[i] else: 0)
        if short > 0:
            ans += short
            for j in 0..i: need[j] += short
    print(ans)
