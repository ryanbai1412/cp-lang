#include <bits/stdc++.h>
using namespace std;
long long seed = 1;
long long nxt(long long n) { seed = seed * 48271 % 2147483647; return seed % n; }
const int N = 5000;
vector<int> head[N];
vector<int> to; vector<long long> lft, cap0;
int main() {
    for (int i = 0; i < 200000; i++) {
        int u = nxt(N), v = nxt(N); long long c = nxt(1000000) + 1;
        head[u].push_back(to.size()); to.push_back(v); lft.push_back(c); cap0.push_back(c);
        head[v].push_back(to.size()); to.push_back(u); lft.push_back(0); cap0.push_back(0);
    }
    int s = 0, t = N - 1; long long total = 0;
    while (true) {
        vector<int> level(N, -1); level[s] = 0; vector<int> q{s};
        for (size_t at = 0; at < q.size(); at++) { int u = q[at];
            for (int a : head[u]) if (lft[a] > 0 && level[to[a]] < 0) { level[to[a]] = level[u] + 1; q.push_back(to[a]); } }
        if (level[t] < 0) break;
        vector<int> next(N, 0);
        while (true) {
            vector<int> path; int u = s; bool dead = false;
            while (u != t) {
                if (next[u] < (int)head[u].size()) { int a = head[u][next[u]]; int v = to[a];
                    if (lft[a] > 0 && level[v] == level[u] + 1) { path.push_back(a); u = v; } else next[u]++; }
                else { if (path.empty()) { dead = true; break; } int a = path.back(); path.pop_back(); u = to[a ^ 1]; next[u]++; }
            }
            if (dead) break;
            long long amt = LLONG_MAX; for (int a : path) amt = min(amt, lft[a]);
            for (int a : path) { lft[a] -= amt; lft[a ^ 1] += amt; }
            total += amt;
        }
    }
    printf("%lld\n", total);
}
