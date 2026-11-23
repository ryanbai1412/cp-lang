// expect: ok, output "2 3 6 7\n1 2 2 3\n8"
// `s.supersets` goes `self` first and the full set last, increasing as masks; `s.subsets` goes down.
domain B = 0..3
let s = Subset<B>.single(B(1))
print(s.supersets.map(g => Int(g)).collect())
print(s.supersets.map(g => g.len).collect())
print(Subset<B>.empty.supersets.collect().len)
