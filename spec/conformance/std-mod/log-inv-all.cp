// expect: ok, output "YES"
type M = Mod<7>
let e: List<M> = []
print(yes(M(3).sqrt() == None and M(3).log(M(6)) == Some(3) and M(3).log(M(1)) == Some(0) and M(2).log(M(3)) == None and M.inv_all([M(1), M(2), M(3)]) == [M(1), M(4), M(5)] and M.inv_all(e) == []))
