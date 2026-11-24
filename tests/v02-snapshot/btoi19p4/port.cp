// Source: Baltic OI 2019 Day 2 Tom's Kitchen, https://dmoj.ca/problem/btoi19p4
// Problem: n meals need A[i] hours each, each from at least k different chefs (positive whole
//   hours each); chef j works at most B[j] hours but is paid for all B[j]. Hire chefs to
//   minimize paid idle hours, or print "Impossible". n, m, k, A[i], B[j] <= 300.
// Solution: impossible if some A[i] < k. Else a set of chefs works iff its hours cover sum A and
//   its sum of min(B[j], n) reaches n * k (a chef gives each meal at least an hour). 0/1
//   knapsack over total paid hours keeping the largest sum of min(B[j], n); the answer is the
//   least total >= sum A that reaches n * k, minus sum A. O(m * sum B).
// original: corpus/submissions/4456166.cpp
let n: 1..=300 = read()
let m: 1..=300 = read()
let k: 1..=300 = read()
domain Meal = 0..n
domain Chef = 0..m
let need: [Meal]1..=300 = read()
let hours: [Chef]1..=300 = read()

domain Paid = 0..=hours.sum()

fn least_waste() -> ?Int:
    if any(a < k for a in need): return None
    var variety: [Paid]?Int = None
    variety[Paid.first!] = 0
    for b in hours:
        for p in Paid.rev:
            if let q = Paid.at(p - b):
                if let v = variety[q]: variety[p] max= v + min(b, n)
    let work = need.sum()
    for p in Paid:
        if p.idx >= work:
            if let v = variety[p]:
                if v >= n * k: return p.idx - work
    None

if let waste = least_waste(): print(waste)
else: print("Impossible")
