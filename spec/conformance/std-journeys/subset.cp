// expect: ok, output "16\n4\n4\n1 3"
// Every `Subset` journey from std-api/R13 is one short call.
domain B = 0..4
// journey: walk all subsets
print(Subset<B>.full.subsets.count())
let s = Subset<B>.single(B(1)).with(B(3))
// journey: walk submasks and supermasks
print(s.subsets.count())
print(s.supersets.count())
// journey: add and remove a member
let t = s.with(B(2)).without(B(2))
// journey: members as a list
print(t.items.collect())
