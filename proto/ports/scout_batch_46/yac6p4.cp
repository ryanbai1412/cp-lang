// YAC 6 P4 — original: corpus/submissions/5149666.cpp
// Idiomatic CP Lang v2 (locked spec: docs/one-page-cheat-sheet.md)

// Grundy values of pile sizes 0..=100, precomputed offline by the original author
// FRICTION 46.5: no spec'd literal for a const array over a written range
const GRUNDY: [0..=100]Int = [
    0, 0, 1, 2, 2, 7, 3, 5, 4, 5, 10, 9, 9, 15, 5, 12, 14,
    17, 15, 17, 17, 23, 24, 27, 22, 34, 18, 28, 41, 29, 21, 34, 33, 41,
    26, 55, 30, 42, 35, 39, 34, 56, 65, 39, 41, 47, 45, 43, 40, 40, 29,
    44, 58, 56, 52, 57, 55, 54, 55, 48, 61, 60, 60, 58, 44, 67, 67, 64,
    64, 37, 66, 64, 74, 70, 72, 47, 66, 75, 74, 73, 90, 81, 79, 80, 78,
    81, 87, 95, 95, 83, 124, 92, 93, 86, 67, 91, 97, 88, 97, 100, 103]

let (n, m): (1..=200_000, 1..=100) = read()
let a: [0..n]1..=100 = read()

// FRICTION 46.6: sum(it) has no xor counterpart, so the nim-sum is a loop
var need = 0
for x in a: need ^= GRUNDY[x]

print(first(i for i in 1..=m if GRUNDY[i] == need), none: -1)
