// expect: ok, output "1,2\n1/2\nYes\nOK"
// rule: 09.1.1
with io(sep: ",", answers: ("Yes", "No")):
    print(1, 2)
    print(1, 2, sep: "/")
    print(yes(true))
    print(yes(true, answers: ("OK", "FAIL")))
