// expect: ok, output "YES"
let c = Counter.of(['a', 'b', 'a'])
print(yes(c.get('a') == 2 and c.get('c') == 0))
