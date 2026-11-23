// expect: ok, output "4 2 W N E N\nE YES N YES S"
// rule: 04.2.20
// A fieldless enum is a domain, so it is the `Seq` of its variants; before, `D.count(p)`, `D.find` and `D.filter` didn't resolve and `D.count` failed in `cpl run`.
enum D = N | E | S | W
print(D.count, D.count(d => d != D.N and d != D.S), D.last!, D.find(d => d != D.E)!, D.filter(d => d == D.E)[0], D.min(by: d => Int(d))!)
print(D.N.next!, yes(D.W.next == None), D.E.prev!, yes(D.N.prev == None), D.S.next!.prev!)
