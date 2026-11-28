// expect: ok, output "5 6 3"
// rule: 02.6.1 12.4.6
// `ns[i].kid[j] = v` and `ns[i].cnt op= v` write the element the list lends, not a copy.
struct N:
    var cnt: Int
    var kid: [0..2]?Int
var ns: List<N> = [N(0, [None, None]), N(3, [None, None])]
ns[0].kid[1] = 5
ns[0].cnt = 4
ns[0].cnt += 2
print(if let c = ns[0].kid[1]: c else: -1, ns[0].cnt, ns[1].cnt)
