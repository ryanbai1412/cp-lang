// expect: ok, output "3"
// rule: 04.4.28
print((1..=5).bsearch(x => x >= 3))
