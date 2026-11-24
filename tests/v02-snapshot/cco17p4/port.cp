// Source: CCO 2017 P4 (Rainfall Storage), https://dmoj.ca/problem/cco17p4
// Problem: n <= 500 pillars of heights 1..50 can be arranged in any order and rain fills
//   every dip. Print, increasing, every total volume of water some arrangement holds.
// Solution: place pillars from tallest to shortest. A new pillar is no taller than anything
//   standing, so it changes no existing level, and its own water level can be any height
//   already placed except the single tallest. So the volumes are a subset sum: each pillar
//   after the first two adds 0 or (an earlier height - its own). O(n * 50 * 50n) bit DP.
// original: corpus/submissions/4575277.cpp
let n: 1..=500 = read()
domain Pillar = 0..n
let height: [Pillar]1..=50 = read()
domain Volume = 0..=50 * n

fn volumes(height: [Pillar]1..=50) -> List<Int>:
    let rising = height.sorted()
    let tall = [rising[n - 1 - i] for i in 0..n]
    var reach: [Volume]Bool = false
    reach[Volume.first!] = true
    var most = 0  // largest reachable volume so far
    for i in 1..n:
        var next = reach.clone()  // level = own height: holds nothing
        var top = most
        for j in 1..i:
            if j == 1 or tall[j] != tall[j - 1]:
                let rise = tall[j] - tall[i]
                for v in 0..=most:
                    if reach[Volume(v)]: next[Volume(v + rise)] = true
                top max= most + rise
        reach = next
        most = top
    [v.idx for v in Volume if reach[v]]

// Every order, with the textbook prefix-max / suffix-max water count.
@alt(volumes)
fn volumes_by_orders(height: [Pillar]1..=50) -> List<Int>:
    var seen: [Volume]Bool = false
    var order: List<Int> = []
    var used: [Pillar]Bool = false
    fn place():
        if order.len == n:
            var total = 0
            for i in 0..n:
                let left = [order[j] for j in 0..=i].max()!
                let right = [order[j] for j in i..n].max()!
                total += min(left, right) - order[i]
            seen[Volume(total)] = true
            return
        for p in Pillar:
            if not used[p]:
                used[p] = true
                order.push(height[p])
                place()
                order.pop()
                used[p] = false
    place()
    [v.idx for v in Volume if seen[v]]

print(volumes(height))
