#include "bits/extc++.h"

using namespace std;

template <typename T>
void dbgh(const T& t) {
    cerr << t << endl;
}

template <typename T, typename... U>
void dbgh(const T& t, const U&... u) {
    cerr << t << " | ";
    dbgh(u...);
}

#ifdef DEBUG
#define dbg(...)                                              \
    cerr << "L" << __LINE__ << " [" << #__VA_ARGS__ << "]: "; \
    dbgh(__VA_ARGS__);
#else
#define dbg(...)
#define cerr   \
    if (false) \
    cerr
#endif

#define endl "\n"
#define long int64_t
#define sz(x) int((x).size())

const int maxn = 1e5 + 5;

int dist[maxn];
vector<int> vis;
vector<pair<int, int>> graph[maxn];

void dfs(int u, int p, int d) {
    dist[u] = d;
    vis.push_back(u);
    for (auto& [v, w] : graph[u]) {
        if (v != p) {
            dfs(v, u, d + w);
        }
    }
}

void dfs(int u) {
    vis.clear();
    dfs(u, -1, 0);
}

pair<int, int> farthest() {
    pair<int, int> opt {-1, 0};
    for (auto& a : vis) {
        opt = max(opt, pair<int, int> {dist[a], a});
    }
    return opt;
}

extern "C" int travelTime(int n,
                          int m,
                          int k,
                          int arru[],
                          int arrv[],
                          int arrw[]) {
    for (int i = 0; i < m; i++) {
        int u = arru[i], v = arrv[i], w = arrw[i];
        graph[u].emplace_back(v, w);
        graph[v].emplace_back(u, w);
    }
    int ans = 0;
    bool cvis[n] {};
    int d1[n], d2[n];
    vector<int> comps;
    for (int i = 0; i < n; i++) {
        if (cvis[i]) {
            continue;
        }
        dfs(i);
        int u = farthest().second;
        for (auto& a : vis) {
            cvis[a] = true;
        }
        dfs(u);
        auto [diam, v] = farthest();
        ans = max(ans, diam);
        for (auto& a : vis) {
            d1[a] = dist[a];
        }
        dfs(v);
        for (auto& a : vis) {
            d2[a] = dist[a];
        }
        int mn = 2e9;
        for (auto& a : vis) {
            mn = min(mn, max(d1[a], d2[a]));
        }
        comps.push_back(mn);
        dbg(mn);
    }
    sort(begin(comps), end(comps), greater<>());
    if (sz(comps) >= 2) {
        ans = max(ans, k + comps[0] + comps[1]);
    }
    if (sz(comps) >= 3) {
        ans = max(ans, 2 * k + comps[1] + comps[2]);
    }
    return ans;
}