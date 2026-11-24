// Source: CCO 2017 P4 (Rainfall Storage), https://dmoj.ca/problem/cco17p4
// limits: 1000 ms, 1024 MB
// Problem: n <= 500 pillars of heights 1..50 can be arranged in any order and rain fills
//   every dip. Print, increasing, every total volume of water some arrangement holds.
// Solution: place pillars from tallest to shortest. A new pillar is no taller than anything
//   standing, so it changes no existing level, and its own water level can be any height
//   already placed except the single tallest. So the volumes are a subset sum: each pillar
//   after the first adds (an earlier non-tallest height - its own), 0 included: a Bitset DP
//   with one union_shifted per distinct earlier height, O(n * 50 * 50n / 64).
// original: corpus/submissions/4575277.cpp
// Bounds: n: 1..=500; height: 1..=50
let n: 1..=500 = read()
domain Pillar = 0..n
type Height = 1..=50
let height: [Pillar]Height = read()
domain Volume = 0..=50 * n

fn volumes(height: [Pillar]Height) -> List<Volume>:
    let tall = height.sorted().rev.collect()
    var reach = Bitset<Volume>.single(Volume.first!)
    var levels: List<Height> = []  // the distinct heights placed after the tallest
    for i in 1..n:
        let before = reach.clone()
        for h in levels: reach.union_shifted(before, h - tall[i])
        if levels.last != Some(tall[i]): levels.push(tall[i])
    reach.each().collect()

// Every order, with the textbook prefix-max / suffix-max water count.
@alt(volumes)
fn volumes_by_orders(height: [Pillar]Height) -> List<Volume>:
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
    [v for v in Volume if seen[v]]

print(volumes(height))
