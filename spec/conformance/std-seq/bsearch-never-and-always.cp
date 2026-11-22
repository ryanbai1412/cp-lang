// expect: ok, output "-1 0 YES YES"
let r = 0..100
print(r.first_true(x => x > 1000) ?? -1, r.first_true(x => x >= 0) ?? -1,
      yes(r.bsearch(x => x > 1000).r == 100), yes(r.bsearch(x => x >= 0).r == 0))
