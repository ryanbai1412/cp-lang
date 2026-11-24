// expect: ok, output "2\n0\n5\na 2\nb 5"
// Every `HashMap`/`Counter` journey from std-api/R13 is one short call.
// journey: count occurrences
var c = ['a', 'b', 'c', 'a'].tally()
print(c.get('a'))
// journey: read with a default
print(c.get('z'))
// journey: add to a value
c.add('b', 4)
print(c.get('b'))
// journey: walk in a fixed order
print(c.items().filter((k, _) => k != 'c').sorted())
