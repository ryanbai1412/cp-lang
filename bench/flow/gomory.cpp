#include <bits/stdc++.h>
using namespace std;
long long seed = 1;
long long nxt(long long n) { seed = seed * 48271 % 2147483647; return seed % n; }
#include "_dinic.inc"
const int N = 500;
int main() {
    Dinic g(N);
    for (int i = 0; i < 5000; i++) { int u = nxt(N), v = nxt(N); long long w = nxt(100) + 1; g.add_both(u, v, w); }
    vector<int> parent(N, 0); long long sum = 0;
    for (int i = 1; i < N; i++) {
        g.reset(); long long w = g.max_flow(i, parent[i]); auto side = g.min_cut(i);
        for (int j = i + 1; j < N; j++) if (side[j] && parent[j] == parent[i]) parent[j] = i;
        sum += w;
    }
    printf("%lld\n", sum);
}
