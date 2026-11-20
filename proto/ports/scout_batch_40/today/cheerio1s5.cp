// Cheerio Contest 1 S5, clearing blocks around chosen H's (buildable today-form)
// original: corpus/submissions/5339756.cpp
let s: [domain S]Char = read()
let k: 0..=1000 = read()
let n = S.len
var head: List<Bool> = []
for i in S: head.push(s[i] == 'H')
var heads: [0..=n]Int = 0
for i in 0..n: heads[i + 1] = heads[i] + Int(head[i])

// clear[l, r]: best score from the block l..r, every pick splitting what is left
var clear: [0..=n, 0..=n]Int = 0
for l in (0..n).rev:
    for r in l + 1..=n:
        for i in l..r:
            if head[i]: clear[l, r] max= r - l + clear[l, i] + clear[i + 1, r]

// best[i, used]: best score from s[i..] with `used` H's already inside chosen blocks
var best: [0..=n, 0..=k]Int = 0
for i in (0..n).rev:
    for used in 0..=k:
        best[i, used] = best[i + 1, used]
        for j in i + 1..=n:
            let now = used + heads[j] - heads[i]
            if now <= k: best[i, used] max= clear[i, j] + best[j, now]
print(best[0, 0])
