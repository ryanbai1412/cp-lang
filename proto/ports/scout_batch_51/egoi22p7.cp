// EGOI '22 P7 (Toy Design): reconstruct connected components via persistent DSU queries.
// A divide-and-conquer segment tree keeps a graph version uniting each segment into one
// component. Merging halves uses Marks<Pin> (O(1) clear) for seen left/right DSU roots
// instead of O(n) linear scans, and binary-searches the right subtree for each match.
// original: corpus/submissions/5203797.cpp

@grader fn Connected(a: Int, i: Int, j: Int) -> Int
@grader fn DescribeDesign(result: List<(Int, Int)>)

@grader
fn ToyDesign(n: Int, max_ops: Int):
    domain Pin = 0..n
    var dsu = Dsu<Pin>()
    var ver: [0..4 * n]Int = 0
    var rep: [0..4 * n]Pin = Pin.first
    var seen_l = Marks<Pin>()
    var seen_r = Marks<Pin>()

    fn query(a: Int, u: Pin, v: Pin) -> Int:
        Connected(a, u.to_idx + 1, v.to_idx + 1)

    fn find_match(o: Int, r: Range<Pin>, x: Pin) -> Pin:
        if r.len == 1: return r.first!
        let m = r.mid!
        let (lc, rc) = (o * 2, o * 2 + 1)
        let (left, right) = (r.l..m.r, m.r..r.r)
        if not any(dsu.find(u) not in seen_r for u in left):
            return find_match(rc, right, x)
        if not any(dsu.find(u) not in seen_r for u in right):
            return find_match(lc, left, x)
        if query(ver[lc], x, rep[lc]) == ver[lc]:
            find_match(lc, left, x)
        else:
            find_match(rc, right, x)

    fn build(o: Int, r: Range<Pin>):
        if r.len == 1:
            ver[o] = 0
            rep[o] = r.first!
            return
        let m = r.mid!
        let (lc, rc) = (o * 2, o * 2 + 1)
        let (left, right) = (r.l..m.r, m.r..r.r)
        build(lc, left)
        build(rc, right)
        seen_l.clear()
        seen_r.clear()
        ver[o] = ver[rc]
        for i in left:
            if not seen_l.add(dsu.find(i)): continue
            let cv = query(ver[o], i, rep[rc])
            if cv != ver[o]:
                ver[o] = cv
                continue
            let rv = find_match(rc, right, i)
            dsu.unite(i, rv)
            let root = dsu.find(i)
            seen_l.add(root)
            seen_r.add(root)
        rep[o] = rep[lc]

    build(1, Pin.first..=Pin.last)
    DescribeDesign([(dsu.find(i).to_idx + 1, i.to_idx + 1) for i in Pin if dsu.find(i) != i])
