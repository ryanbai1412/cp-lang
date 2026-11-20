// AAC 5 P4 — original: corpus/submissions/4239900.cpp
// Idiomatic CP Lang v2 (locked spec: docs/one-page-cheat-sheet.md)

let (n, x): (1..=1_000_000, 0..=1_000_000_000) = read()

fn size() -> Int:
    if n == 1: return 0
    if n % 2 == 0: return n / 2
    let h = (n - 3) / 2
    let flip = h % 4 == 0 or h % 4 == 3
    n / 2 + Int(flip != (x % 2 == 1))

let ans = size()
print(ans)
for i in 1..=ans: print(i)
