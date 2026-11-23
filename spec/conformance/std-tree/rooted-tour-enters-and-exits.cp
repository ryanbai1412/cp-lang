// expect: ok, output "6 3 3"
domain V = 0..3
let adj: [V]List<V> = [[1], [0, 2], [1]]
let t = Rooted(adj, root: 0)
var (enters, exits) = (0, 0)
for e in t.tour:
    match e:
        Enter(_, _) => enters += 1
        _ => exits += 1
print(t.tour.count(), enters, exits)
