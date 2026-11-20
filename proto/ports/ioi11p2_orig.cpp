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
#define dbg(...)                                           \
    cerr << "L" << __LINE__ << " [" << #__VA_ARGS__ << "]" \
         << ": ";                                          \
    dbgh(__VA_ARGS__)
#else
#define cerr   \
    if (false) \
    cerr
#define dbg(...)
#endif

#define endl "\n"
#define long int64_t
#define sz(x) int(std::size(x))

constexpr int maxn = 2e5 + 5;

int kv, ans, depth1[maxn];
long depth2[maxn];
vector<pair<int, int>> graph[maxn];

void pdfs(int u, int d1 = 0, long d2 = 0) {
    dbg(u, d1, d2);
    depth1[u] = d1;
    depth2[u] = d2;
    for (auto& [v, w] : graph[u]) {
        graph[v].erase(
            find(begin(graph[v]), end(graph[v]), pair<int, int> {u, w}));
        pdfs(v, d1 + 1, d2 + w);
    }
}

map<long, int> dfs(int u) {
    map<long, int> cur {{depth2[u], depth1[u]}};
    for (auto& [v, _] : graph[u]) {
        auto&& cv = dfs(v);
        if (sz(cv) > sz(cur)) {
            swap(cv, cur);
        }
        for (auto& [d2, d1] : cv) {
            auto it = cur.find(kv + 2 * depth2[u] - d2);
            if (it != cur.end()) {
                ans = min(ans, d1 + it->second - 2 * depth1[u]);
            }
        }
        for (auto& [d2, d1] : cv) {
            auto it = cur.emplace(d2, d1).first;
            it->second = min(it->second, d1);
        }
    }
    return cur;
}

int best_path(int n, int _kv, int edges[][2], int edgews[]) {
    kv = _kv;
    for (int i = 0; i < n - 1; i++) {
        auto& [u, v] = edges[i];
        int w = edgews[i];
        graph[u].emplace_back(v, w);
        graph[v].emplace_back(u, w);
    }
    pdfs(0);
    ans = 1e9;
    dfs(0);
    if (ans == 1e9) {
        return -1;
    } else {
        return ans;
    }
}
// The original is the function the IOI grader calls. This `main` stands in for
// the grader so the port can be stressed against it: n k, then n - 1 roads.
static int roads_in[200005][2], lens_in[200005];
int main() {
	int n, k;
	if (scanf("%d %d", &n, &k) != 2) return 1;
	for (int i = 0; i + 1 < n; i++)
		if (scanf("%d %d %d", &roads_in[i][0], &roads_in[i][1], &lens_in[i]) != 3) return 1;
	printf("%d\n", best_path(n, k, roads_in, lens_in));
}
