// Source: IOI 1999 Day 2 P3 (IOI '99 P6), A Strip of Land, https://dmoj.ca/problem/ioi99p6
// Problem: a U x V grid of heights (U, V <= 700, |h| <= 30000) and a limit C <= 10. Print the
//   largest-area rectangle at most 100 squares wide (west-east) whose highest and lowest
//   heights differ by at most C, as Xmin Ymin Xmax Ymax (y grows to the north, the first
//   input row is y = V).
// Solution: Fix the left column and widen right by up to 100 columns, keeping each row's min
//   and max over the column span. For each span, two pointers over the rows with monotone
//   queues for the max of row maxima and the min of row minima find, for every top row, the
//   lowest bottom row that keeps the spread <= C. O(U * 100 * V).
// Buildable today-form.
// original: corpus/submissions/4899527.cpp
let m: 1..=700 = read()
let n: 1..=700 = read()
let k: 0..=10 = read()
let h: [0..n * m]Int = read()

var best = -1
var ans: List<Int> = [0, 0, 0, 0]
var lo: [0..n]Int = 0
var hi: [0..n]Int = 0
// monotone queues of row numbers over the current row window top..bottom
var q_hi: [0..n]Int = 0
var q_lo: [0..n]Int = 0
for left in 0..m:
    for r in 0..n:
        lo[r] = h[r * m + left]
        hi[r] = lo[r]
    for right in left..min(m, left + 100):
        for r in 0..n:
            lo[r] min= h[r * m + right]
            hi[r] max= h[r * m + right]
        var (hi_head, hi_tail, lo_head, lo_tail) = (0, 0, 0, 0)
        var bottom = 0  // one past the last row of the window
        for top in 0..n:
            bottom max= top
            while bottom < n:
                let peak = if hi_head < hi_tail: max(hi[q_hi[hi_head]], hi[bottom]) else: hi[bottom]
                let pit = if lo_head < lo_tail: min(lo[q_lo[lo_head]], lo[bottom]) else: lo[bottom]
                if peak - pit > k: break
                while hi_head < hi_tail and hi[q_hi[hi_tail - 1]] <= hi[bottom]: hi_tail -= 1
                q_hi[hi_tail] = bottom
                hi_tail += 1
                while lo_head < lo_tail and lo[q_lo[lo_tail - 1]] >= lo[bottom]: lo_tail -= 1
                q_lo[lo_tail] = bottom
                lo_tail += 1
                bottom += 1
            let area = (bottom - top) * (right - left + 1)
            if area > best:
                best = area
                ans = [left + 1, n - bottom + 1, right + 1, n - top]
            if hi_head < hi_tail and q_hi[hi_head] == top: hi_head += 1
            if lo_head < lo_tail and q_lo[lo_head] == top: lo_head += 1
print(ans)
