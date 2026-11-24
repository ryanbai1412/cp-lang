// expect: ok, output "3 6\nYES YES"
// rule: 10.2.10
// input: "3 1\n\n2  \n"
var xs = List<Int>()
while let x = read<?Int>():
    xs.push(x)
print(xs.len, xs.sum())
let pair: ?(Int, Int) = read()
let word = read<?Str>()
print(yes(pair is None), yes(word is None))
