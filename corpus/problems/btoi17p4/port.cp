// Source: Baltic OI 2017 Day 2 (Friends), https://dmoj.ca/problem/btoi17p4
// limits: 1000 ms, 1024 MB
// Problem: n <= 2500 students and their claimed friend lists (total <= 30000), p + q <= 15.
//   Print "detention" unless friendship is symmetric and the students split into groups of
//   1..p students, each with at most q friend pairs leaving the group; then print "home", the
//   number of groups and each group (size, members).
// Solution: no valid student has over p + q - 1 friends. For each student u a bounded search
//   tree finds some valid group containing u: take the first undecided friend of the group and
//   branch on including it or cutting it, at most p members and q cut pairs, so about 2^(p+q)
//   leaves. Then uncross: for each pair i < j, A_i - A_j stays valid or else A_j - A_i does (cut
//   size is submodular), so after all pairs the groups are disjoint. O(n^2 (p + q)) after search.
// original: corpus/submissions/4523876.cpp
// Bounds: n: 1..=2500; p: 1..=16; q: 0..=16; friends: 0..n
let n: 1..=2500 = read()
let p: 1..=16 = read()
let q: 0..=16 = read()
domain V = 0..n
let friends: [V]List<V> = read()

// The search's state: who is in the group, and each student's side once
// decided (true: in the group, false: cut off from it).
var members: List<V> = []
var side: [V]?Bool = None

// Some valid group holding `members`: branch on the first undecided friend
// of the group, taking it in or cutting it off.
fn grow() -> ?List<V>:
    if members.len > p: return None
    if count(side[v] == Some(false) for a in members for v in friends[a]) > q: return None
    let open = [v for a in members for v in friends[a] if side[v] == None].first
    let v = open else: return Some(members.sorted())
    side[v] = Some(true)
    members.push(v)
    let inside = grow()
    members.pop()
    side[v] = Some(false)
    let found = if inside != None: inside else: grow()
    side[v] = None
    found

fn group_of(u: V) -> ?List<V>:
    members = [u]
    side[u] = Some(true)
    let found = grow()
    side[u] = None
    found

var stamp: [V]Int = 0
var now = 0
fn mark(a: List<V>):
    now += 1
    for x in a: stamp[x] = now

fn minus(a: List<V>, b: List<V>) -> List<V>:
    mark(b)
    [x for x in a if stamp[x] != now]

fn valid(a: List<V>) -> Bool:
    if a.len > p: return false
    mark(a)
    count(stamp[v] != now for u in a for v in friends[u]) <= q

// A valid group per student, uncrossed pair by pair into disjoint groups.
fn groups() -> ?[V]List<V>:
    if not V.all(u => friends[u].len < p + q and friends[u].all(v => friends[v].contains(u))):
        return None
    var blocks: [V]List<V> = []
    for u in V:
        let b = group_of(u) else: return None
        blocks[u] = b
    for i in V:
        for j in V.after(i):
            let cut = minus(blocks[i], blocks[j])
            if valid(cut): blocks[i] = cut
            else: blocks[j] = minus(blocks[j], blocks[i])
    Some(blocks)

if let blocks = groups():
    let used = [u for u in V if not blocks[u].is_empty]
    print("home")
    print(used.len)
    for u in used: print(blocks[u].len, blocks[u])
else: print("detention")
