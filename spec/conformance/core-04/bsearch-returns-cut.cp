// expect: ok, output "8"
// rule: 04.4.20
// `r.bsearch(ok)` gives the cut where `ok` turns true, which always exists (rebuild 7, H-BSEARCH-NAME).
let c = (0..10).bsearch(x => x * x >= 50)
print(Int(c))
