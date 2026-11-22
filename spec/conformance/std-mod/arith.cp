// expect: ok, output "1 5 7 3 6 3 4 0"
type M = Mod<7>
print(M(3) * M(5), M(3).inv(), Mod<10>(3).inv(), M.from_int(10), M.from_int(-8), M(2).sqrt()!, Mod<1_000_000_007>(16).sqrt()!, M(0).sqrt()!)
