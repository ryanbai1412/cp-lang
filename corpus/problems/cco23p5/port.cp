// Source: CCO 2023 P5 (Travelling Trader), https://dmoj.ca/problem/cco23p5
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
//   O(n log n) for sorting children.
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

// The best plan at u naming one child per kind in `kinds`, each among the kind's top 3, all distinct.
fn named(u: City, kinds: List<(done: Bool, back: Bool)>) -> ?(gain: Int, kids: List<City>):
    var picks: List<(gain: Int, kids: List<City>)> = [(0, [])]
    for kd in kinds:
        picks = [(p.gain + extra(v, kd.done, kd.back), p.kids + [v]) for p in picks for v in top3(u, kd.done, kd.back) if not p.kids.contains(v)]
    picks.max(by: p => p.gain)

@memo(array)
fn top3(u: City, done: Bool, back: Bool) -> List<City> =
    kids[u].sorted(by: v => desc(extra(v, done, back))).take(3).collect()

fn extra(v: City, done: Bool, back: Bool) -> Int = best(v, done, back) - profit[v]

// The plans at u worth comparing: the best of each shape.
fn plans(u: City, done: Bool, back: Bool) -> List<(gain: Int, parts: List<Part>)>:
    let rest = kids[u].map(v => profit[v]).sum()
    var all: List<(gain: Int, parts: List<Part>)> = [(0, [Here])]
    fn one(d: Bool, b: Bool) -> ?City = top3(u, d, b).first
    if not back:
        if let v = kids[u].max(by: v => best(v, false, false)): all.push((best(v, false, false), [Here, Kid(v, false, false)]))
        if let p = named(u, [(false, true), (true, false)]):
            all.push((rest + p.gain, [Here, Kid(p.kids[0], false, true), Rest, Kid(p.kids[1], true, false)]))
        if not done:
            if let p = named(u, [(true, true), (false, true), (true, false)]):
                all.push((rest + p.gain, [Kid(p.kids[0], true, true), Here, Kid(p.kids[1], false, true), Rest, Kid(p.kids[2], true, false)]))
            if let p = named(u, [(true, true), (false, false)]):
                all.push((rest + p.gain, [Rest, Kid(p.kids[0], true, true), Here, Kid(p.kids[1], false, false)]))
            if let v = one(false, false): all.push((rest + extra(v, false, false), [Rest, Here, Kid(v, false, false)]))
    elif done:
        if let v = one(false, true): all.push((rest + extra(v, false, true), [Here, Kid(v, false, true), Rest]))
    else:
        if let v = one(true, true): all.push((rest + extra(v, true, true), [Rest, Kid(v, true, true), Here]))
    all

@memo(array)
fn best(u: City, done: Bool, back: Bool) -> Int = profit[u] + plans(u, done, back).map(p => p.gain).max()!

fn replay(u: City, done: Bool, back: Bool):
    let plan = plans(u, done, back).max(by: p => p.gain)!
    let named_kids = [v for part in plan.parts if let Kid(v, _, _) = part]
    for part in plan.parts:
        match part:
            Here => route.push(u)
            Rest => route += [v for v in t.children(u) if not named_kids.contains(v)]
            Kid(v, d, b) => replay(v, d, b)

let root = City.first!
match k:
    1 => print(down(root)); descend(root)
    2 => print(best(root, true, false)); replay(root, true, false)
    _ => print(profit.sum()); sweep(root, true)
print(route.len)
print(route)
