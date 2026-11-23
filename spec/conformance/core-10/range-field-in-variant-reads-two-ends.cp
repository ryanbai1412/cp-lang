// expect: ok, output "3 7"
// rule: 10.2.3
// input: "1 2 4 7\n"
// A `Range<V>` field inside an enum variant reads two inclusive ends (rebuild round 6 H-TAG-RANGE-READ).
domain V = 1..=5
enum Q:
    @tag(1) Add(r: Range<V>, x: Int)
    @tag(2) Get(v: V)
let q = read<Q>()
match q:
    Add(r, x): print(r.len, x)
    Get(v): print(Int(v))
