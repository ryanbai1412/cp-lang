// BOI 2018 "Alternating Current": wires cover wrapping arcs of a ring of n
// segments; give each wire a direction so both directions cover the ring.
let (n, m) = read<(3..=1e5, 2..=1e5)>()
domain Seg = 1..=n
domain Wire = [m]
let arc = read<[Wire]RingRange<Seg>>()

let nest = Seg.ring.nesting(arc)
let tops = nest.tops
let k = tops.len

fn covers(arcs: Seq<RingRange<Seg>>, part: RingRange<Seg>) -> Bool =
    Seg.ring.coverage(arcs).min_over(part) > 0

fn spread(top_dir: tops.keys -> Bool) -> ?[Wire]Bool:
    let dir = [Wire] w => if nest.is_top(w): top_dir(nest.top(w)) else: not top_dir(nest.top(w))
    let both = [false, true].all(d => covers(Wire.filter(w => dir[w] == d).map(w => arc[w]), Seg.ring.all))
    both.then(dir)

let answer: ?[Wire]Bool =
    if k % 2 == 0 or k == 1:
        spread(t => t.is_odd())
    elif k <= 3:
        tops.keys.find_map(s => spread(t => tops.ring.dist(s, t).is_even()))
    elif not covers(arc.values, Seg.ring.all):
        None
    else:
        let alone = [tops.keys] t => covers(chain(nest.inside(t).map(w => arc[w]), [arc[tops.ring.prev(t)], arc[tops.ring.next(t)]]), arc[t])
        let good = alone.count()
        tops.ring.windows(4).find_map(w =>
            let (a, b, c, d) = (w[0], w[1], w[2], w[3])
            let pair = chain(nest.inside(b), nest.inside(c)).map(x => arc[x])
            if good - alone[b].int - alone[c].int == k - 2 and covers(chain(pair, [arc[a], arc[d]]), arc[b].hull(arc[c])):
                spread(t => if t == c: tops.ring.dist(b, t).is_odd() else: tops.ring.dist(b, t).is_even())
            else: None)

match answer:
    Some(dir) => print(dir.map(d => d.int), sep: "")
    None => print("impossible")
