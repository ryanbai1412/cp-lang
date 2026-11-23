use std.graph.{complement, components, reachable}
// egoi23p7 Sopsug: N - 1 tubes forming a tree directed toward one root, using every existing
// tube and no banned pair, or NO. The existing tubes must form a forest; each of its trees is a
// block whose root needs one new tube into another block. Block c may hang under block k unless
// c's root is banned from every vertex of k, so the blocks need a spanning arborescence of the
// complement of those "cannot" arcs. Its root can only be where the last search of a full
// search forest starts; then each block hangs under the first block reached before it that may
// take it, and the root is paid for once per "cannot" arc skipped.
let (n, m, k) = read<(2..=300_000, 0..=300_000, 0..=300_000)>()
domain V = [n]
let existing = read<[m](from: V, to: V)>()
let banned = read<[k](from: V, to: V)>()

fn solve() -> ?List<(from: V, to: V)>:
    var up: [V]?V = None
    for t in existing:
        if up[t.from] != None:
            return None
        up[t.from] = Some(t.to)
    let forest = Rooted.try_from_parents(up) else: return None
    let (domain B, top) = compress(forest.roots)
    let block = [V] v => top.at(forest.root_of(v))
    let members = block.buckets()
    let bans_from = banned.map(.from).buckets()
    var cannot: [B]List<B> = []
    for c in B:
        for (k, hits) in bans_from[top.get(c)].tally(by: i => block[banned[i].to]).items:
            if k != c and hits == members[k].len:
                cannot[k].push(c)
    let blocked = Set([(k, c) for k in B for c in cannot[k]])
    let hang = complement(cannot)
    // A full search forest's last tree is started by the only block that can reach all others.
    let (domain C, comp) = components(hang)
    let root = B.find(b => comp[b] == C.last!)!
    let order = reachable(hang, from: [root]).collect()
    if order.len < B.count:
        return None
    let ban = Set(banned)
    var out = List(existing)
    for b in order.drop(1):
        let k = order.find(x => x != b and (x, b) not in blocked)!
        let c = top.get(b)
        out.push((from: c, to: members[k].find(x => (from: c, to: x) not in ban)!))
    out

if let tubes = solve():
    for t in tubes:
        print(t.from, t.to)
else:
    print("NO")
