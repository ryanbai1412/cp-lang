// expect: E0318
// rule: 01.7.3
// `less` is a binding, not the variant `Less`, so it matches every Ordering and the arms after it never run.
let x = match cmp(2, 1):
    less => "less"
    greater => "greater" // <- here
    equal => "equal"
print(x)
