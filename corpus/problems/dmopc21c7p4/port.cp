// Source: DMOPC '21 Contest 7 P4 (Rocket Race), https://dmoj.ca/problem/dmopc21c7p4
// limits: 2000 ms, 256 MB
// Problem: n <= 2e5 single-use rockets; rocket i moves a_i forward, then b_i back. Q <= 2e5
//   events: "0 r" prints the fewest rockets that reach or pass r metres at some moment (going
//   forward only), or -1; "t x y" replaces rocket t by (x, y). Values <= 1e9.
// Solution: k rockets reach their best peak by using k - 1 of the largest nets a - b, then the
//   one with the largest a among the rest, or the chosen ones with the largest b last. Every
//   version of a rocket (initial or replacement) gets a fixed slot by decreasing net, so a
//   SegTree over slots keeps each fleet's largest a and b, last net, count and net sum; a binary
//   search over prefixes of the positive nets finds the shortest that reaches r, else all the
//   positives plus the best other a. A replacement moves its rocket to the new version's slot.
//   O(log^2 n) per query.
// original: corpus/submissions/4825739.cpp
// Bounds: n, q: 1..=2e5; initial.a, initial.b, a, b, r: 1..=1e9; t: 0..=n

// the rockets in a run of slots, folded left to right
struct Fleet(top_a: Int, top_b: Int, last_net: Int, count: Int, net: Int):
    pub fn of(r: (a: Int, b: Int)) -> Fleet = Fleet(r.a, r.b, r.a - r.b, 1, r.a - r.b)

let fleets = Monoid<Fleet>(
    id: Fleet(0, 0, 0, 0, 0),
    op: (x, y) => Fleet(
        max(x.top_a, y.top_a),
        max(x.top_b, y.top_b),
        if y.count > 0: y.last_net else: x.last_net,
        x.count + y.count,
        x.net + y.net,
    ),
)

enum Event:
    Ask(r: Int)
    Swap(t: Rocket, to: Int)

let n: 1..=2e5 = read()
domain Rocket = 1..=n
let initial: [Rocket](a: 1..=1e9, b: 1..=1e9) = read()
var versions: List<(a: Int, b: Int)> = [(a: x.a, b: x.b) for x in initial]
let q: 1..=2e5 = read()
var events: List<Event> = []
for _ in 0..q:
    let t: 0..=n = read()
    if t == 0:
        let r: 1..=1e9 = read()
        events.push(Ask(r))
    else:
        let (a, b): (1..=1e9, 1..=1e9) = read()
        events.push(Swap(Rocket(t), versions.len))
        versions.push((a:, b:))

domain Version = 0..versions.len
domain Slot = 0..versions.len
var slot: [Version]Slot = Slot.first!
for (s, v) in Version.sort(by: v => desc(versions[v.ord].a - versions[v.ord].b)).items:
    slot[v] = Slot.nth(s)
let positive = versions.count(v => v.a > v.b)

var now: [Rocket]Version = [Rocket] t => Version.nth(t.ord)
var fleet = SegTree<Slot, Fleet>(fleets, [Slot] _ => fleets.id)
for t in Rocket: fleet.set(slot[now[t]], Fleet.of(versions[now[t].ord]))

fn split(k: Int) -> (used: Fleet, rest: Fleet) =
    (used: fleet.fold(..Slot.cuts.nth(k)), rest: fleet.fold(Slot.cuts.nth(k)..))

fn reaches(k: Int, r: Int) -> Bool:
    let (used, rest) = split(k)
    used.net + used.top_b >= r or used.net - used.last_net + rest.top_a >= r

fn fewest(r: Int) -> ?Int:
    if let k = (0..=positive).first_true(k => reaches(k, r)): return max(1, split(k).used.count)
    let (used, rest) = split(positive)
    if used.net + rest.top_a >= r: Some(used.count + 1) else: None

for e in events:
    match e:
        Ask(r) => print(fewest(r) ?? -1)
        Swap(t, to) =>
            fleet.set(slot[now[t]], fleets.id)
            now[t] = Version.nth(to)
            fleet.set(slot[now[t]], Fleet.of(versions[to]))
