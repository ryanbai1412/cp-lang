// Source: IOI 1999 Day 2 P3 (IOI '99 P6), A Strip of Land, https://dmoj.ca/problem/ioi99p6
// Problem: a U x V grid of heights (U, V <= 700, |h| <= 30000) and a limit C <= 10. Print the
//   largest-area rectangle at most 100 squares wide (west-east) whose highest and lowest
//   heights differ by at most C, as Xmin Ymin Xmax Ymax (y grows to the north, the first
//   input row is y = V).
// Solution: Fix the left column and widen right by up to 100 columns, keeping each row's min
//   and max over the column span. For each span, two pointers over the rows with monotone
//   queues for the max of row maxima and the min of row minima find, for every top row, the
//   lowest bottom row that keeps the spread <= C. O(U * 100 * V).
// original: corpus/submissions/4899527.cpp
let (m, n, k): (1..=700, 1..=700, 0..=10) = read()
domain Col = 0..m
domain Row = 0..n
let h: [Row, Col]-30_000..=30_000 = read()

var best: ?(area: Int, corner: List<Int>) = None
for left in Col:
    var lo: [Row]Int = [Row] r => h[r, left]
    var hi: [Row]Int = [Row] r => h[r, left]
    for right in left..=Col(min(m - 1, Int(left) + 99)):
        for r in Row:
            lo[r] min= h[r, right]
            hi[r] max= h[r, right]
        // rows top..bottom, the tallest window from each top whose spread stays within k
        var peak = Window(max)
        var pit = Window(min)
        var bottom = 0
        for top in Row:
            bottom max= Int(top)
            while let r = Row.at(bottom) and max(peak.all ?? hi[r], hi[r]) - min(pit.all ?? lo[r], lo[r]) <= k:
                peak.push(hi[r])
                pit.push(lo[r])
                bottom += 1
            let area = (bottom - Int(top)) * (Int(right) - Int(left) + 1)
            if best == None or area > best!.area:
                best = Some((area:, corner: [Int(left) + 1, n - bottom + 1, Int(right) + 1, n - Int(top)]))
            if Int(top) < bottom:
                peak.pop(hi[top])
                pit.pop(lo[top])
print(best!.corner)
