// COCI 2020/21 Contest 6 #5 Index: n papers in a row with citation counts,
// and q questions. A question l r asks for the h-index of the papers
// l..=r: the largest h such that at least h of them have h or more
// citations.
//
// Mo's algorithm: answer the questions in an order that moves the two ends
// of the window O(n sqrt q) steps in total. The original keeps the window
// in a Fenwick tree and binary-searches h for every question. The h-index
// moves by at most one when a paper enters or leaves, so the window below
// keeps h itself, at O(1) a step.
let (n, q): (Int, Int) = read()
domain Paper = 1..=n
domain Ask = 0..q
let cites: [Paper]Int = read()
let asks: [Ask](l: Paper, r: Paper) = read()

domain Count = 0..=n                // a paper cited more than n times counts as n
var have: [Count]Int = 0            // papers in the window, by citations
var (h, strong) = (0, 0)            // strong: papers in the window cited at least h times

fn enter(p: Paper):
    let c = min(cites[p], n)
    have[Count(c)] += 1
    if c >= h: strong += 1
    if strong - have[Count(h)] > h:
        strong -= have[Count(h)]
        h += 1

fn leave(p: Paper):
    let c = min(cites[p], n)
    have[Count(c)] -= 1
    if c >= h: strong -= 1
    if strong < h:
        h -= 1
        strong += have[Count(h)]

let block = max(1, isqrt(n))
fn mo_key(a: Ask) -> (Int, Int):
    let (b, r) = (asks[a].l.to_idx / block, asks[a].r.to_idx)
    (b, if b % 2 == 0: r else: -r)

var answer: [Ask]Int = 0
var (lo, hi) = (0, 0)               // the window is the papers at offsets lo..hi
for a in sorted(Ask, by: mo_key):
    let (l, r) = (asks[a].l.to_idx, asks[a].r.to_idx + 1)
    while hi < r:
        enter(Paper.nth(hi))
        hi += 1
    while lo > l:
        lo -= 1
        enter(Paper.nth(lo))
    while hi > r:
        hi -= 1
        leave(Paper.nth(hi))
    while lo < l:
        leave(Paper.nth(lo))
        lo += 1
    answer[a] = h
for a in Ask: print(answer[a])
