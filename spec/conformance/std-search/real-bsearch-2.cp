// expect: ok, output "YES"
print(yes((0.0..1.0).bsearch(x => x > 2.0, steps: 100) == None))
