#include <bits/stdc++.h>
using namespace std;
int main() {
    int n, m;
    scanf("%d %d", &n, &m);
    vector<string> g(n);
    for (auto& row : g) {
        static char buf[200005];
        scanf("%s", buf);
        row = buf;
    }
    if (n > m) {
        vector<string> t(m, string(n, ' '));
        for (int i = 0; i < n; i++)
            for (int j = 0; j < m; j++) t[j][i] = g[i][j];
        g = t;
        swap(n, m);
    }
    mt19937_64 rng(20201231);
    vector<uint64_t> w(n);
    for (auto& x : w) x = rng();
    long long total = 0;
    vector<uint64_t> h(m), run;
    for (int r1 = 0; r1 < n; r1++) {
        fill(h.begin(), h.end(), 0);
        for (int r2 = r1; r2 < n; r2++) {
            for (int c = 0; c < m; c++)
                if (g[r2][c] == 'Y') h[c] += w[r2];
            if (r2 == r1) continue;
            for (int c = 0; c < m;) {
                if (g[r1][c] != g[r2][c]) { c++; continue; }
                run.clear();
                while (c < m && g[r1][c] == g[r2][c]) run.push_back(h[c++]);
                sort(run.begin(), run.end());
                for (size_t i = 0, j; i < run.size(); i = j) {
                    for (j = i; j < run.size() && run[j] == run[i]; j++) {}
                    total += (long long)(j - i) * (j - i - 1) / 2;
                }
            }
        }
    }
    printf("%lld\n", total);
}
