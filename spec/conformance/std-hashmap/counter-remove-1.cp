// expect: ok, output "YES"
var c = Counter.of(['a', 'a'])
print(yes(c.remove('a') == Some(2) and c.remove('a') == None))
