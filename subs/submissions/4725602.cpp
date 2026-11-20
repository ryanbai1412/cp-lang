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
 
const int maxn = 1005;
 
int t, ans[maxn];
vector<int> graph[maxn];
 
void dfs(int u, int p = -1, int d = 0) {
    if (!(d & 1)) {
        ans[u] = t++;
    }
    for (auto& v : graph[u]) {
        if (v != p) {
            dfs(v, u, d + 1);
        }
    }
    if (d & 1) {
        ans[u] = t++;
    }
    dbg(u, ans[u]);
}
 
vector<int> label(int n, int, vector<int> u, vector<int> v) {
    for (int i = 0; i < n; i++) {
        graph[i].clear();
    }
    for (int i = 0; i < n - 1; i++) {
        graph[u[i]].push_back(v[i]);
        graph[v[i]].push_back(u[i]);
    }
    t = 0;
    dfs(0);
    return vector<int>(ans, ans + n);
}
 
int find_next_station(int s, int t, vector<int> c) {
    assert(s != t);
    if (sz(c) == 1) {
        return c[0];
    }
    if (!s) {
        return *lower_bound(begin(c), end(c), t);
    }
    if (s < c[0]) {
        // I am tin, others are tout
        if (s <= t && t <= c[sz(c) - 2]) {
            return *lower_bound(begin(c), end(c), t);
        } else {
            return c.back();
        }
    } else {
        // I am tout, others are tin
        if (c[1] <= t && t <= s) {
            return *(--upper_bound(begin(c), end(c), t));
        } else {
            return c[0];
        }
    }
}