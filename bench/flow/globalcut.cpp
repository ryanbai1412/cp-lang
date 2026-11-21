#include <bits/stdc++.h>
using namespace std;
long long seed = 1;
long long nxt(long long n) { seed = seed * 48271 % 2147483647; return seed % n; }
const int N = 600;
int main() {
    int n = N; vector<long long> mat(n * n);
    for (int i = 0; i < 20000; i++) { int a = nxt(N), b = nxt(N); long long w = nxt(100) + 1; if (a != b) { mat[a * n + b] += w; mat[b * n + a] += w; } }
    vector<vector<int>> group(n); for (int i = 0; i < n; i++) group[i] = {i};
    vector<char> alive(n, 1), added(n); vector<long long> key(n); long long best = LLONG_MAX; vector<int> best_group;
    for (int phase = 1; phase < n; phase++) {
        fill(key.begin(), key.end(), 0); fill(added.begin(), added.end(), 0);
        int s = 0, t = 0;
        for (int step = 0; step < n - phase + 1; step++) {
            int u = -1; for (int i = 0; i < n; i++) if (alive[i] && !added[i] && (u < 0 || key[i] > key[u])) u = i;
            added[u] = 1; s = t; t = u;
            for (int i = 0; i < n; i++) if (alive[i] && !added[i]) key[i] += mat[t * n + i];
        }
        if (key[t] < best) { best = key[t]; best_group = group[t]; }
        for (int x : vector<int>(group[t])) group[s].push_back(x);
        alive[t] = 0;
        for (int i = 0; i < n; i++) { mat[s * n + i] += mat[t * n + i]; mat[i * n + s] = mat[s * n + i]; }
    }
    printf("%lld %zu\n", best, best_group.size());
}
