// expect: ok, output "mid\nlow"
// rule: 01.10.4
// `elif c:` tries each condition in turn and runs the first that holds, in block and one-line forms.
fn band(x: Int) -> Str:
    if x > 10: "high"
    elif x > 5: "mid"
    else: "low"
print(band(7))
let y = 2
if y > 10: print("high") elif y > 5: print("mid") else: print("low")
