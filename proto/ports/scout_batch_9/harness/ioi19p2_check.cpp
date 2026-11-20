// Checker for ioi19p2: any split with the right sizes and at least two
// connected parts is accepted; all zeros only when the original also gives up.
// Usage: check <input> <port's output> <original's output>; exit 0 if accepted.
#include <bits/stdc++.h>
using namespace std;

int main(int, char** argv) {
    ifstream in(argv[1]), ours(argv[2]), ref(argv[3]);
    int n, m, want[4];
    in >> n >> m >> want[1] >> want[2];
    want[3] = n - want[1] - want[2];
    vector<int> a(n), b(n), p(n);
    for (auto& x : a) if (!(ours >> x) || x < 0 || x > 3) return puts("bad label"), 1;
    for (auto& x : b) ref >> x;
    bool gave_up = count(a.begin(), a.end(), 0) == n;
    if (gave_up != (count(b.begin(), b.end(), 0) == n)) return puts("verdicts differ"), 1;
    if (gave_up) return 0;
    iota(p.begin(), p.end(), 0);
    function<int(int)> find = [&](int u) { return p[u] == u ? u : p[u] = find(p[u]); };
    while (m--) {
        int u, v;
        in >> u >> v;
        if (a[u] == a[v]) p[find(u)] = find(v);
    }
    int connected = 0;
    for (int k = 1; k <= 3; k++) {
        if (count(a.begin(), a.end(), k) != want[k]) return printf("part %d has the wrong size\n", k), 1;
        set<int> roots;
        for (int i = 0; i < n; i++) if (a[i] == k) roots.insert(find(i));
        connected += roots.size() == 1;
    }
    if (connected < 2) return puts("fewer than two connected parts"), 1;
    return 0;
}
