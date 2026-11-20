// Checker for cmt21r2ztr: any numbers that fit every fact are accepted.
// Usage: check <input> <port's output> <original's output>; exit 0 if accepted.
#include <bits/stdc++.h>
using namespace std;

int main(int, char** argv) {
    ifstream in(argv[1]), ours(argv[2]), ref(argv[3]);
    string a, b;
    ours >> a;
    ref >> b;
    if (a != b) return puts("verdicts differ"), 1;
    if (a == "NO") return 0;
    int n, m;
    in >> n >> m;
    vector<long long> v(n + 1);
    for (int i = 1; i <= n; i++) {
        if (!(ours >> v[i]) || v[i] < 1 || v[i] > 1000000000) return puts("number missing or out of range"), 1;
    }
    while (m--) {
        int i, j, k, x;
        in >> i >> j >> k >> x;
        long long t[3] = {v[i], v[j], v[k]};
        sort(t, t + 3);
        if (t[1] != x) return printf("median of %d %d %d is %lld, not %d\n", i, j, k, t[1], x), 1;
    }
    return 0;
}
