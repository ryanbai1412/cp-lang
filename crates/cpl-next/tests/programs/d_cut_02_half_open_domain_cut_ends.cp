// expect: ok, output "YES 0\n3 YES\n1 2"
// rule: R04.2.32 R04.3.2
// A cut of `0..4` has no position before its first cut or after its last.
domain V = 0..4
print(yes(V.cuts(0).l == None), Int(V.cuts(0).r!))
print(Int(V.cuts(4).l!), yes(V.cuts(4).r == None))
print(Int(V.cuts(2).l!), Int(V.cuts(2).r!))
