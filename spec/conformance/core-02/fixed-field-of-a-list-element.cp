// expect: E0405
// rule: 02.6.1
// A fixed field of a list's element is fixed too: `ns[0].cnt = v` assigns the field.
struct N:
    cnt: Int
var ns: List<N> = [N(0)]
ns[0].cnt = 4 // <- here
