// Source: CCO 2023 P5 (Travelling Trader), https://dmoj.ca/problem/cco23p5
// limits: 2000 ms, 1024 MB
// Problem: a tree of n <= 2e5 cities with profits p_i <= 1e9, and K in 1..=3. The trader does
//   business in city 1, walks one road per day and may go at most K days in a row without doing
//   business in a new city. Print the max total profit, then how many cities and which, in order.
// Solution: root at city 1. K = 1: the route is a downward path, the best root-to-leaf sum.
//   K = 3: every city fits: list even depths in preorder and odd depths in postorder, so
//   consecutive cities are at most 3 roads apart. K = 2: best(u, done, back) (done: u is visited
//   on arrival; back: the route must end next to u's parent) is profit[u] plus the best plan at u.
//   A plan is an order of parts: u itself, the children not named (each a 1-step side trip, so
//   all of their profits count) and up to 3 named children, each with its own (done, back);
//   a named child of each kind is among that kind's top 3. The route replays the best plans.
//   The plans fill children before parents (t.order.rev); O(n).
// Bounds: n: 1..=2e5; k: 1..=3; profit: 1..=1e9
let (n, k): (1..=2e5, 1..=3) = read()
domain City = 1..=n
let t = Rooted(Graph.undirected(read<Tree<City>>()), root: City.first!)
let profit: [City]1..=1e9 = read()
let kids = [City] u => t.children(u).collect()
var route: List<City> = []

@memo(array)
fn down(u: City) -> Int = profit[u] + (t.children(u).map(down).max() ?? 0)

fn descend(u: City):
    route.push(u)
    if let v = t.children(u).max(by: down): descend(v)

fn sweep(u: City, pre: Bool):
    if pre: route.push(u)
    for v in t.children(u): sweep(v, not pre)
    if not pre: route.push(u)

// A part of the route at a city: the city, its other children's side trips, or a named child.
enum Part = Here | Rest | Kid(v: City, done: Bool, back: Bool)

// Which part a named child plays: is it visited on arrival, must its route end next to u?
type Kind = (done: Bool, back: Bool)

// A kind with its top 3 children at a city, best first.
type Ranked = (kd: Kind, top: List<City>)

fn ranked(u: City, kd: Kind) -> Ranked = (kd, top3(u, kd.done, kd.back))

// Every kind ranked at a city.
type Tops = (ff: Ranked, ft: Ranked, tf: Ranked, tt: Ranked)

fn tops(u: City) -> Tops = (
    ranked(u, (false, false)),
    ranked(u, (false, true)),
    ranked(u, (true, false)),
    ranked(u, (true, true)),
)

fn gain_of(v: City, kd: Kind) -> Int = extra(v, kd.done, kd.back)

// The best two distinct named children of kinds a and b, each among its kind's top 3.
fn pick2(a: Ranked, b: Ranked) -> ?(gain: Int, x: City, y: City):
    var found: ?(gain: Int, x: City, y: City) = None
    for x in a.top:
        for y in b.top:
            let gain = gain_of(x, a.kd) + gain_of(y, b.kd)
            if y != x and (found.map(f => gain > f.gain) ?? true): found = Some((gain, x, y))
    found

// The same for three distinct named children of kinds a, b and c.
fn pick3(a: Ranked, b: Ranked, c: Ranked) -> ?(gain: Int, x: City, y: City, z: City):
    var found: ?(gain: Int, x: City, y: City, z: City) = None
    for x in a.top:
        for y in b.top:
            for z in c.top:
                let gain = gain_of(x, a.kd) + gain_of(y, b.kd) + gain_of(z, c.kd)
                if x != y and y != z and x != z and (found.map(f => gain > f.gain) ?? true):
                    found = Some((gain, x, y, z))
    found

// The 3 children of u with the most extra, best first (ties to the earlier child), in one pass.
fn top3(u: City, done: Bool, back: Bool) -> List<City>:
    var top: List<City> = []
    for v in kids[u]:
        let gain = extra(v, done, back)
        var i = top.len
        top.push(v)
        while i > 0 and extra(top[i - 1], done, back) < gain:
            top[i] = top[i - 1]
            i -= 1
        top[i] = v
        if top.len > 3: top.pop()
    top

fn extra(v: City, done: Bool, back: Bool) -> Int = best(v, done, back) - profit[v]

// The best plan at u: its gain, and its parts when `keep` (the route needs them only at the
// cities it replays, so `best` builds none).
fn plan(u: City, tp: Tops, done: Bool, back: Bool, keep: Bool) -> (gain: Int, parts: List<Part>):
    let rest = kids[u].map(v => profit[v]).sum()
    var top: (gain: Int, parts: List<Part>) = (0, if keep: [Here] else: [])
    fn offer(gain: Int, parts: List<Part>):
        if gain > top.gain: top = (gain, parts)
    if not back:
        if let v = kids[u].max(by: v => best(v, false, false)): offer(
            best(v, false, false),
            if keep: [Here, Kid(v, false, false)] else: [],
        )
        if let p = pick2(tp.ft, tp.tf):
            offer(
                rest + p.gain,
                if keep: [Here, Kid(p.x, false, true), Rest, Kid(p.y, true, false)] else: [],
            )
        if not done:
            if let p = pick3(tp.tt, tp.ft, tp.tf):
                offer(
                    rest + p.gain,
                    if keep: [
                        Kid(p.x, true, true),
                        Here,
                        Kid(p.y, false, true),
                        Rest,
                        Kid(p.z, true, false),
                    ] else: [],
                )
            if let p = pick2(tp.tt, tp.ff):
                offer(
                    rest + p.gain,
                    if keep: [Rest, Kid(p.x, true, true), Here, Kid(p.y, false, false)] else: [],
                )
            if let v = tp.ff.top.first: offer(
                rest + extra(v, false, false),
                if keep: [Rest, Here, Kid(v, false, false)] else: [],
            )
    elif done:
        if let v = tp.ft.top.first: offer(
            rest + extra(v, false, true),
            if keep: [Here, Kid(v, false, true), Rest] else: [],
        )
    else:
        if let v = tp.tt.top.first: offer(
            rest + extra(v, true, true),
            if keep: [Rest, Kid(v, true, true), Here] else: [],
        )
    top

// The best gains of every city's four plans, filled children before parents.
var bests = [City] _ => (ff: 0, ft: 0, tf: 0, tt: 0)

fn best(u: City, done: Bool, back: Bool) -> Int:
    let b = bests[u]
    if done: (if back: b.tt else: b.tf) else: (if back: b.ft else: b.ff)

fn fill():
    for u in t.order.rev:
        let tp = tops(u)
        fn at(done: Bool, back: Bool) -> Int = profit[u] + plan(u, tp, done, back, false).gain
        bests[u] = (at(false, false), at(false, true), at(true, false), at(true, true))

fn replay(u: City, done: Bool, back: Bool):
    let chosen = plan(u, tops(u), done, back, true)
    let named_kids = [v for part in chosen.parts if let Kid(v, _, _) = part]
    for part in chosen.parts:
        match part:
            Here => route.push(u)
            Rest => route += [v for v in t.children(u) if not named_kids.contains(v)]
            Kid(v, d, b) => replay(v, d, b)

let root = City.first!
match k:
    1 => print(down(root)); descend(root)
    2 => fill(); print(best(root, true, false)); replay(root, true, false)
    _ => print(profit.sum()); sweep(root, true)
print(route.len)
print(route)
