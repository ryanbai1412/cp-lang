// expect: E0410
// rule: 01.7.6
// A `continue` inside an `if` that is only an operand has no statement to leave from.
for i in 0..3:
    print(i + (if i == 1:
        continue  // <- here
    else:
        i
    ))
