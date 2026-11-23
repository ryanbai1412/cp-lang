// expect: ok
// rule: judge.2
// tool: build --judge atcoder -o -
// has: "For judge `atcoder`: g++"
// lacks: "For judge `codeforces`"
// lacks: "--stack=268435456"
// The second line names only the chosen profile, never another profile's command or stack.
print(1)
