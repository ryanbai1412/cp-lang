// IOI '14 P1 (Rail): reconstruct positions and directions of n stations from pairwise
// shortest-path queries via an extern @grader oracle.
// Sort stations by distance from station 0, maintain leftmost/rightmost placed endpoints
// (l, r: ?Station) and a Map<Int, Bool> of placed positions (true = D/type-2, false = C/type-1).
// original: corpus/submissions/4691718.cpp

@grader fn getDistance(i: Int, j: Int) -> Int

@grader
fn findLocation(n: Int, first: Int) -> (List<Int>, List<Int>):
    domain S = 0..n
    var dist: [S]Int = 0
    for i in S if i != S.first: dist[i] = getDistance(0, i.to_idx)
    let ord = sorted([i for i in S if i != S.first], by: i => dist[i])

    var loc: [S]Int = 0
    var kind: [S]Int = 0
    loc[S.first] = first
    kind[S.first] = 1

    var placed = Map<Int, Bool>()
    placed[first] = false
    var l = S.first
    var r: ?S = none
    var open_r = first

    for a in ord:
        if r == none:
            r = a
            loc[a] = first + dist[a]
            kind[a] = 2
            placed[loc[a]] = true
            continue
        let rv = r!
        let ql = getDistance(l.to_idx, a.to_idx)
        let qr = getDistance(rv.to_idx, a.to_idx)
        let pos_l = loc[l] + ql
        let pos_r = loc[rv] - qr
        let mid = (loc[rv] + pos_l - qr) / 2
        let is_right = if let is_d = placed.get(mid): not is_d else: mid > open_r
        if is_right:
            loc[a] = pos_l
            kind[a] = 2
            if loc[a] > loc[rv]: r = a
        else:
            loc[a] = pos_r
            kind[a] = 1
            if loc[a] < loc[l]: l = a
            open_r max= loc[a]
        placed[loc[a]] = kind[a] == 2

    ([loc[S(i)] for i in 0..n], [kind[S(i)] for i in 0..n])
