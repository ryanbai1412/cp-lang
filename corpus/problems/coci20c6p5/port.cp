// Source: COCI 2020/2021 Contest 6 P5 (Index), https://dmoj.ca/problem/coci20c6p5
// Problem: n <= 2e5 papers with citation counts p_i <= 2e5, and q <= 2e5 questions l r: print
//   the h-index of papers l..r, the largest h with at least h of them cited at least h times.
// Solution: Mo's algorithm (blocks of sqrt n, alternating right-end order). Adding or removing a
//   paper changes h by at most one, so the window keeps counts by citations (capped at n), h and
//   how many papers have at least h citations, fixing h in O(1) per step: O((n + q) sqrt n).
let (n, q): (Int, Int) = read()
domain Paper = 1..=n
domain Ask = 0..q
let cites: [Paper]Int = read()
let asks: [Ask](l: Paper, r: Paper) = read()

domain Count = 0..=n      // a paper cited more than n times counts as n
var have: [Count]Int = 0  // papers in the window, by citations
var (h, strong) = (0, 0)  // strong: papers in the window cited at least h times

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
var (lo, hi) = (0, 0)  // the window is the papers at offsets lo..hi
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
