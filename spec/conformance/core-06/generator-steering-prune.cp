// expect: ok, output "3"
// rule: 06.3.4 07.6.3
print([1, 2].subsequences().filter(s => s.items.len <= 1 or { s.prune(); false }).count())
