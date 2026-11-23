// cco20p4: every pair of n towns is joined by a red or a blue road. For each start, print a
// route of the minimum length n: every town once, switching colour at most once.
// Insert the towns one by one, the start last, into a path of blue roads then red roads,
// keeping both junction roads in their part; then read the path from the start.
let n = read<2..=2_000>()
domain V = 1..=n
let rows = read<[n - 1]Str>()  // rows[v.ord - 1] lists the roads from v to every earlier town
fn is_red(u: V, v: V) -> Bool:
    let (lo, hi) = (min(u, v), max(u, v))
    rows[hi.ord - 1][lo.ord] == 'R'

for s in V:
    let order = [V.nth((s.ord + k) % n) for k in 1..=n]
    var blue: Deque<V> = Deque()
    var red: Deque<V> = Deque()
    let pair = [order[0], order[1]]
    if is_red(order[0], order[1]): red = Deque.of(pair) else: blue = Deque.of(pair)
    for u in order[2..]:
        if red.is_empty:
            let v = blue.last!
            if is_red(v, u):
                blue.pop()
                red.push_front(v)
            if is_red(u, blue.first!): red.push(u) else: blue.push_front(u)
        elif is_red(u, red.first!):
            red.push_front(u)
            if let w = blue.last:
                if is_red(w, u):
                    blue.pop()
                    red.push_front(w)
            elif not is_red(red.last!, u):
                blue.push(red.pop()!)
        else:
            blue.push(red.pop_front()!)
            if is_red(u, red.first ?? blue.first!): red.push_front(u) else: blue.push(u)
    var path = blue.to_list() + red.to_list()
    if blue.last == Some(s) or red.last == Some(s): path.reverse()
    print(n)
    print(path.rotated_left(path.keys.find(i => path[i] == s)!))
