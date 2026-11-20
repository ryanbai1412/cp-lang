// TLE '17 Contest 5 P1, one-question interactive
// original: corpus/submissions/5034414.cpp
// The judge answers "guess g" with secret - g, so one guess is enough.
let diff: Int = ask("guess", -50)
print("answer", diff - 50)

@interactor
fn judge():
    let secret: -1e9..=1e9 = read()
    let (word, g): (Str, Int) = read_output()
    assert word == "guess"
    print(secret - g)
    let (verdict, x): (Str, Int) = read_output()
    assert verdict == "answer" and x == secret
