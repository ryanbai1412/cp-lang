// expect: ok, output "YES Impossible Yes"
// rule: 10.3.3
// copies: 1
let word = yes(false, answers: ("Possible", "Impossible"))
with io(answers: ("Yes", "No")):
    print("YES {word} {yes(true)}")
