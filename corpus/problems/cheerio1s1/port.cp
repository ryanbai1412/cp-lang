// Source: Cheerio Contest 1 S1/J4 (Going to School), https://dmoj.ca/problem/cheerio1s1
// Problem: Bob walks from 0 to D <= 1e9 at speed 1 past n <= 2e5 lights at increasing points
//   p_i; light i is red for r_i then green for g_i seconds, cycling from t = 0 (all just red),
//   and he waits at a red one. Print his arrival time (64-bit).
// Solution: walk the lights in order, adding each distance to t; at a light, phase = t mod
//   (r + g), and if phase < r he waits r - phase. Answer t + D - p_n: O(n).
// original: corpus/submissions/5337471.cpp
// Bounds: n: 1..=2e5; k: 1..=1e9; lights.p, lights.r, lights.g: 1..=1e9
let n: 1..=2e5 = read()
let k: 1..=1e9 = read()
type Light = (p: 1..=1e9, r: 1..=1e9, g: 1..=1e9)
let lights: [0..n]Light = read()

var pos = 0
var t = 0
for (p, r, g) in lights:
    t += p - pos
    pos = p
    let phase = t % (r + g)
    if phase < r: t += r - phase
print(t + k - pos)
