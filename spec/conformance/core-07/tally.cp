// expect: ok, output "2"
// rule: 07.11.4
print([1, 1, 2].tally(by: x => x)[1])
