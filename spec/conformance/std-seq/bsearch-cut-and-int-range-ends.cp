// expect: ok, output "3\n8\n2 5"
domain P = 0..10
print(P.cuts.bsearch(x => x.ord >= 3).ord)
print((0..100).bsearch(x => x * x >= 50).r)
print((2..5).ends)
