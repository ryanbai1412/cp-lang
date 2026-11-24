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
// The original is the two functions the IOI grader calls. This `main` stands in for the
// grader: n, the n - 1 links, q, then q queries `s t`; it labels the tree once and prints
// the station each query's packet is sent to.
int main() {
	int n;
	if (scanf("%d", &n) != 1) return 1;
	vector<int> u(n - 1), v(n - 1);
	vector<vector<int>> adj(n);
	for (int i = 0; i < n - 1; i++) {
		scanf("%d %d", &u[i], &v[i]);
		adj[u[i]].push_back(v[i]);
		adj[v[i]].push_back(u[i]);
	}
	vector<int> lab = label(n, n - 1, u, v), station(n);
	for (int i = 0; i < n; i++) station[lab[i]] = i;
	int q;
	scanf("%d", &q);
	while (q--) {
		int s, t;
		scanf("%d %d", &s, &t);
		vector<int> c;
		for (int w : adj[s]) c.push_back(lab[w]);
		sort(c.begin(), c.end());
		printf("%d\n", station[find_next_station(lab[s], lab[t], c)]);
	}
}
