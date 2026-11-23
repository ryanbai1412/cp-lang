// Baltic OI 2019 Tom's Kitchen: n meals need need[i] hours of work each, and
// each meal must be worked on by at least k different chefs. A hired chef is
// paid for all of hours[j]. Hire chefs so every meal can be cooked and the
// paid-but-unused hours are as few as possible, or say it is impossible.
// A chef adds variety to at most n meals, an hour each. A set of chefs works
// exactly when its hours cover the total need and its variety hours
// sum(min(hours[j], n)) reach n * k. So: for each total of hired hours, the
// most variety, a 0/1 knapsack.
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
