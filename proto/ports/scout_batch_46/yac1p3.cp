// YAC 1 P3 — original: corpus/submissions/4380584.cpp
// Idiomatic CP Lang v2 (locked spec: docs/one-page-cheat-sheet.md)

let (n, t): (1..=1000, 1..=1000) = read()

// mirror every move through the centre; with n odd, take the centre first
for _ in 0..t:
    if n % 2 == 1:
        print(1)
        print((n + 1) / 2, (n + 1) / 2)
    else:
        print(2)
    // FRICTION 46.4: the spec never says print flushes before a read outside ask()
    for (x, y) in read_until((0, 0)):
        print(n + 1 - x, n + 1 - y)
