// expect: ok, output "3"
// rule: 04.4.1
print(Int((1..=5).bsearch(x => x >= 3)))
