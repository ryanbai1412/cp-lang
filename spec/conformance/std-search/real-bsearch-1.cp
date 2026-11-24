// expect: ok, output "YES"
print(yes((0.0..8.0).bsearch(x => x >= 3.0, steps: 3) == Some(3.0)))
