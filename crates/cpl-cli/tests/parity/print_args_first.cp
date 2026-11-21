var log = 0

fn side(tag: Str) -> Int:
    print(tag)
    log += 1
    log

print("A", side("B"))
print(side("C"), "D", side("E"), log)
print(log, side("F"), log)
let xs = [side("G"), side("H")]
print("xs", xs, side("I"), end: "!\n")
print("plain", log)
