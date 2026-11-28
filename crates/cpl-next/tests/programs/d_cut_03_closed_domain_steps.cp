// expect: ok, output "1 5 YES\nYES YES 3 4"
// rule: R04.2.32 R04.3.1 R04.3.2
// In `1..=5` the walk's ends bound cuts, `next`/`prev` and `offset` alike, at every base.
domain W = 1..=5
print(Int(W.cuts(0).r!), Int(W.cuts(5).l!), yes(W.cuts(5).r == None))
print(yes(W(5).next == None), yes(W(1).prev == None), Int(W(2).next!), Int(W(2).offset(2)!))
