// COCI 2021/22 #2 Kaučuk: n lines `section T`, `subsection T` or
// `subsubsection T`. Print each title behind its number: sections count
// 1, 2, ...; a subsection restarts under each section, and so on ("2.1.3 T").
// original: corpus/submissions/4143397.cpp
let n: 1..=100 = read()
var at: [1..=3]Int = 0
for _ in 0..n:
    let (kind, title): (Str, Str) = read()
    let depth = 1 + (kind.len - "section".len) / "sub".len
    at[depth] += 1
    for d in depth + 1..=3: at[d] = 0
    print(".".join("{at[d]}" for d in 1..=depth), title)
