// expect: exit 1
// rule: fmt.1
// tool: fmt --check
// has: "would reformat"
// lacks: "0 would change"
if true:
  print(1)
