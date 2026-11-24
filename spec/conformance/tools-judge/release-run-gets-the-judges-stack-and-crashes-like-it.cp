// expect: exit 3
// rule: judge.8
// tool: run --release {case}
// file: cp.toml "[judge.nothreads]\nextends = \"generic\"\nstack_mb = 8\nthread_stack_mb = 0\n"
// input: "2000000"
// has: "the program crashed"
// judge: nothreads
// A release run uses the judge's flags and its stack (`ulimit -s`): on a judge that bans threads, recursion two million deep fits in 8 MB or not just as it would there.
fn d(n: Int) -> Int = if n == 0: 0 else: (d(n - 1) * 31 + n) % 1000000007
let n: Int = read()
print(d(n))
