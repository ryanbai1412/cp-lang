// CF 2030D QED's Favorite Permutation: the boundary after i must be crossed iff
// max(p[..=i]) > i; it is blocked iff s_i = L and s_{i+1} = R. Count needed blocked ones.
@!multitest
let (n, q): (3..=2e5, 1..=2e5) = read()
domain V = 1..=n
let p = read<[V]V>()
let s = read<Str of "LR">()
var right = [V] i => s[Int(i) - 1] == 'R'
let pm = p.scan(max)
let need = [V] i => Int(pm[i.r]) > Int(i)

fn blocked(i: V) -> Bool = i < V.last! and need[i] and not right[i] and right[i.next!]

var bad = V.count(blocked)
repeat q:
    let i = read<V>()
    for j in [i.prev!, i]:
        if blocked(j): bad -= 1
    right[i] = not right[i]
    for j in [i.prev!, i]:
        if blocked(j): bad += 1
    print(yes(bad == 0))
