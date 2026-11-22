import { readFileSync } from 'node:fs';
import { join } from 'node:path';
import type { Test } from './course';

export const languages = {
  cpp: { name: 'C++', shiki: 'cpp' },
  python: { name: 'Python', shiki: 'python' },
  java: { name: 'Java', shiki: 'java' },
  rust: { name: 'Rust', shiki: 'rust' },
} as const;
export type Lang = keyof typeof languages;

interface Problem {
  id: string;
  title: string;
  statement: string;
  teaches: string;
  tests: Test[];
  yours: Record<Lang, string>;
  notes: Record<Lang, string[]>;
}

const t = (input: string, expected: string): Test => ({ input, expected });

const problems: Problem[] = [
  {
    id: '1-sum',
    title: 'Sum of a list',
    statement: 'Read n (1 ≤ n ≤ 2·10⁵) and then n integers between -10⁹ and 10⁹. Print their sum.',
    teaches: 'read, ranges as input contracts, print',
    tests: [t('5\n3 1 4 1 5\n', '14\n'), t('1\n-7\n', '-7\n'), t('3\n1000000000 1000000000 1000000000\n', '3000000000\n')],
    yours: {
      cpp: 'int n; cin >> n;\nlong long s = 0;\nfor (int i = 0; i < n; i++) { long long x; cin >> x; s += x; }\ncout << s << "\\n";',
      python: 'n = int(input())\nprint(sum(map(int, input().split())))',
      java: 'int n = in.nextInt();\nlong s = 0;\nfor (int i = 0; i < n; i++) s += in.nextLong();\nSystem.out.println(s);',
      rust: 'let n: usize = it.next().unwrap().parse().unwrap();\nlet s: i64 = (0..n).map(|_| it.next().unwrap().parse::<i64>().unwrap()).sum();\nprintln!("{s}");',
    },
    notes: {
      cpp: ['The range `-1e9..=1e9` is the statement\'s constraint. The compiler proves the sum needs 64 bits, so there is no `long long` to forget.', 'In a debug run, input outside the range traps with the line and the rule.'],
      python: ['Types are written where data enters: at `read` and in function signatures. Everywhere else they are inferred.', 'This compiles to C++, so it runs at C++ speed.'],
      java: ['No `Scanner` or `BufferedReader`: `read()` takes its shape from the declared type.', 'No class or `main` method: a program is its top-level statements.'],
      rust: ['No `unwrap` and `parse` chains: `read()` parses into the declared type.', 'Integer widths come from ranges, not from `i32` or `i64`.'],
    },
  },
  {
    id: '2-pairs',
    title: 'Pairs with a target sum',
    statement: 'Read n and x, then n distinct integers in increasing order. Count the pairs i < j with a[i] + a[j] = x.',
    teaches: 'var, while, two pointers',
    tests: [t('6 10\n1 3 5 7 9 11\n', '2\n'), t('2 3\n1 2\n', '1\n'), t('3 100\n1 2 3\n', '0\n')],
    yours: {
      cpp: 'int i = 0, j = n - 1, pairs = 0;\nwhile (i < j) {\n  long long s = a[i] + a[j];\n  if (s == x) pairs++, i++, j--;\n  else if (s < x) i++;\n  else j--;\n}',
      python: 'i, j, pairs = 0, n - 1, 0\nwhile i < j:\n    s = a[i] + a[j]\n    if s == x: pairs += 1; i += 1; j -= 1\n    elif s < x: i += 1\n    else: j -= 1',
      java: 'int i = 0, j = n - 1, pairs = 0;\nwhile (i < j) {\n  long s = a[i] + a[j];\n  if (s == x) { pairs++; i++; j--; }\n  else if (s < x) i++;\n  else j--;\n}',
      rust: 'let (mut i, mut j, mut pairs) = (0, n - 1, 0);\nwhile i < j {\n    let s = a[i] + a[j];\n    if s == x { pairs += 1; i += 1; j -= 1; }\n    else if s < x { i += 1 } else { j -= 1 }\n}',
    },
    notes: {
      cpp: ['`a[i] + a[j]` can reach 2·10⁹. The compiler knows both bounds, so the sum gets a width that holds it; no overflow by accident.'],
      python: ['`let` is a constant, `var` is a variable. Most names are `let`.', '`elif` and `while` read the same as in Python.'],
      java: ['`var (i, j) = (0, n - 1)` declares two variables at once from a tuple.'],
      rust: ['`var` is `let mut`; `let` is immutable, as in Rust.'],
    },
  },
  {
    id: '3-grid',
    title: 'Nearest exit on a grid',
    statement: "Read an h × w grid of '.', '#', one 'S' and some 'E'. Print the fewest steps from S to any E, moving up, down, left or right, or -1.",
    teaches: 'domains, grids, ?Int and ??',
    tests: [t('3 4\nS..#\n.#..\n...E\n', '5\n'), t('1 3\nS#E\n', '-1\n'), t('1 2\nSE\n', '1\n')],
    yours: {
      cpp: 'vector<vector<int>> dist(h, vector<int>(w, -1));\nqueue<pair<int,int>> q; q.push(s); dist[s.first][s.second] = 0;\nwhile (!q.empty()) { auto [r, c] = q.front(); q.pop();\n  for (auto [dr, dc] : dirs) { int nr = r + dr, nc = c + dc;\n    if (nr < 0 || nr >= h || nc < 0 || nc >= w) continue; ... } }',
      python: 'dist = [[-1] * w for _ in range(h)]\nq = deque([s]); dist[s[0]][s[1]] = 0\nwhile q:\n    r, c = q.popleft()\n    for dr, dc in ((1,0),(-1,0),(0,1),(0,-1)):\n        nr, nc = r + dr, c + dc\n        if 0 <= nr < h and 0 <= nc < w and ...',
      java: 'int[][] dist = new int[h][w];\nfor (int[] row : dist) Arrays.fill(row, -1);\nArrayDeque<int[]> q = new ArrayDeque<>();\n// four directions, bounds checks, ...',
      rust: 'let mut dist = vec![vec![None; w]; h];\nlet mut q = VecDeque::from([s]);\nwhile let Some((r, c)) = q.pop_front() {\n    for (dr, dc) in [(0, 1), (1, 0), (0, -1), (-1, 0)] { /* checked_add, bounds */ }\n}',
    },
    notes: {
      cpp: ['`p.adj4` yields only the neighbours inside the grid. There are no bounds checks to get wrong.', '`?Int` replaces the `-1` sentinel: an unreached cell is `None`, and `??` supplies the answer when there is none.'],
      python: ['`None` and `Some(x)` are checked at compile time: you cannot add 1 to a distance that might be `None`.'],
      java: ['Arrays are indexed by the grid\'s own row and column types, so a row index can\'t be used as a column.'],
      rust: ['`?Int` is `Option<i64>`, but `!` traps with a source-level report instead of a panic backtrace.'],
    },
  },
  {
    id: '4-paths',
    title: 'Count paths in a DAG',
    statement: 'Read a DAG with n vertices and m edges u → v. Print the number of paths from 1 to n modulo 10⁹ + 7.',
    teaches: 'functions, Mod<P>, memoized recursion',
    tests: [t('4 5\n1 2\n1 3\n2 4\n3 4\n2 3\n', '3\n'), t('2 0\n', '0\n'), t('1 0\n', '1\n')],
    yours: {
      cpp: 'const int MOD = 1e9 + 7;\nvector<long long> memo(n + 1, -1);\nfunction<long long(int)> paths = [&](int v) -> long long {\n  if (v == n) return 1;\n  if (memo[v] != -1) return memo[v];\n  long long t = 0;\n  for (int w : adj[v]) t = (t + paths(w)) % MOD;\n  return memo[v] = t;\n};',
      python: '@lru_cache(None)\ndef paths(v):\n    if v == n: return 1\n    return sum(paths(w) for w in adj[v]) % MOD\n# plus sys.setrecursionlimit and threading for deep graphs',
      java: 'long[] memo = new long[n + 1]; Arrays.fill(memo, -1);\nlong paths(int v) {\n  if (v == n) return 1;\n  if (memo[v] != -1) return memo[v];\n  long t = 0;\n  for (int w : adj.get(v)) t = (t + paths(w)) % MOD;\n  return memo[v] = t;\n}',
      rust: 'fn paths(v: usize, adj: &[Vec<usize>], memo: &mut [Option<u64>]) -> u64 {\n    if let Some(x) = memo[v] { return x; }\n    let t = adj[v].iter().map(|&w| paths(w, adj, memo)).sum::<u64>() % MOD;\n    memo[v] = Some(t); t\n}',
    },
    notes: {
      cpp: ['`Mod<1_000_000_007>` does the reduction on every operation; a missing `% MOD` can\'t be written.', 'The memo table is sized from the parameter\'s type `V`. In 0.3 the decorator is spelled `@cache`.'],
      python: ['No recursion limit: recursion depth is limited by the stack, which judge profiles set to the judge\'s size.'],
      java: ['No memo array to allocate and fill with -1: the decorator owns it.'],
      rust: ['No `&mut memo` threaded through every call.'],
    },
  },
  {
    id: '5-rmq',
    title: 'Range minimum with updates',
    statement: 'Read n values and q queries: "1 i x" sets a[i] = x, "2 l r" prints the minimum of a[l..r].',
    teaches: 'the library and operation values',
    tests: [t('5 4\n5 3 8 1 9\n2 1 3\n1 2 0\n2 1 3\n2 4 5\n', '3\n0\n1\n'), t('1 1\n7\n2 1 1\n', '7\n')],
    yours: {
      cpp: 'struct SegTree { int n; vector<int> t;\n  void set(int i, int x) { for (t[i += n] = x; i > 1; i >>= 1) t[i >> 1] = min(t[i], t[i ^ 1]); }\n  int query(int l, int r) { /* half-open, 0-based */ }\n};',
      python: '# Usually a hand-written iterative segment tree, and often too slow in CPython.',
      java: 'int[] t = new int[2 * n];\nvoid set(int i, int x) { for (t[i += n] = x; i > 1; i >>= 1) t[i >> 1] = Math.min(t[i], t[i ^ 1]); }',
      rust: '// A hand-rolled segment tree struct, or a crate the judge may not have.',
    },
    notes: {
      cpp: ['The tree is indexed by `V`, the same 1-based positions as the input: no `l - 1` conversions.', '`l..=r` is inclusive and says so.'],
      python: ['`SegTree` is in the standard library and compiles to a flat array.'],
      java: ['The operation is a value: `Monoid<Int>(identity, op)`.'],
      rust: ['Generic over the operation without traits or impl blocks.'],
    },
  },
  {
    id: '6-multitest',
    title: 'Several test cases',
    statement: 'Problem 3 again, but the input starts with t, the number of test cases.',
    teaches: '@!multitest and why state can\'t leak',
    tests: [t('2\n3 4\nS..#\n.#..\n...E\n1 3\nS#E\n', '5\n-1\n')],
    yours: {
      cpp: 'int t; cin >> t;\nwhile (t--) {\n  // every global array must be cleared here, or the last test leaks into this one\n}',
      python: 'for _ in range(int(input())):\n    solve()',
      java: 'int t = in.nextInt();\nwhile (t-- > 0) solve();',
      rust: 'for _ in 0..t { solve(&mut it); }',
    },
    notes: {
      cpp: ['`@!multitest` reads t and runs the rest of the file once per test. Everything below it is fresh each time, so state can\'t leak.'],
      python: ['The single-test and multitest programs differ by one line.'],
      java: ['No `solve()` method to extract.'],
      rust: ['No state struct to reset.'],
    },
  },
];

export interface QuickProblem extends Problem {
  source: string;
}

export function quickstart(): QuickProblem[] {
  return problems.map((p) => ({
    ...p,
    source: readFileSync(join(process.cwd(), 'content/quickstart', `${p.id}.cp`), 'utf8'),
  }));
}
