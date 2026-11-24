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
 
// The original calls the IOI grader's build. This stand-in keeps the matrix it is
// given, and `main` reads n and the matrix p, then prints 0, or 1 and the bridges.
static std::vector<std::vector<int>> built;
void build(std::vector<std::vector<int>> b) { built = b; }
 
int construct(vector<vector<int>> arr) {
    for (auto& a : arr) {
        for (auto& b : a) {
            if (b == 3) {
                return 0;
            }
        }
    }
    int n = sz(arr);
    bool vis[n] {};
    vector<vector<int>> trees;
    for (int i = 0; i < n; i++) {
        if (vis[i]) {
            continue;
        }
        vis[i] = true;
        trees.push_back({i});
        for (int j = 0; j < n; j++) {
            if (!vis[j] && arr[i][j] == 1) {
                trees.back().push_back(j);
                vis[j] = true;
            }
        }
    }
    vector<vector<int>> cycles;
    memset(vis, 0, sizeof(vis));
    for (int i = 0; i < sz(trees); i++) {
        if (vis[i]) {
            continue;
        }
        vis[i] = true;
        cycles.push_back({i});
        for (int j = 0; j < sz(trees); j++) {
            if (!vis[j] && arr[trees[i][0]][trees[j][0]] == 2) {
                cycles.back().push_back(j);
                vis[j] = true;
            }
        }
        if (sz(cycles.back()) == 2) {
            return 0;
        }
    }
    int rtree[n], rcycle1[n], rcycle[n];
    for (int i = 0; i < sz(trees); i++) {
        for (auto& a : trees[i]) {
            rtree[a] = i;
        }
    }
    for (int i = 0; i < sz(cycles); i++) {
        for (auto& a : cycles[i]) {
            rcycle1[a] = i;
        }
    }
    for (int i = 0; i < n; i++) {
        rcycle[i] = rcycle1[rtree[i]];
    }
    for (int i = 0; i < n; i++) {
        for (int j = 0; j < n; j++) {
            int x;
            if (rtree[i] == rtree[j]) {
                x = 1;
            } else if (rcycle[i] == rcycle[j]) {
                x = 2;
            } else {
                x = 0;
            }
            if (arr[i][j] != x) {
                return 0;
            }
        }
    }
    vector<vector<int>> graph(n, vector<int>(n));
    auto add_edge = [&](int u, int v) -> void {
        graph[u][v] = graph[v][u] = true;
    };
    for (auto& a : trees) {
        for (int i = 1; i < sz(a); i++) {
            add_edge(a[0], a[i]);
        }
    }
    for (auto& a : cycles) {
        if (sz(a) == 1) {
            continue;
        }
        for (int i = 0; i < sz(a); i++) {
            add_edge(trees[a[i]][0], trees[a[(i + 1) % sz(a)]][0]);
        }
    }
    build(graph);
    return 1;
}
int main() {
	int n;
	if (scanf("%d", &n) != 1) return 1;
	vector<vector<int>> p(n, vector<int>(n));
	for (auto& row : p)
		for (auto& x : row) scanf("%d", &x);
	if (!construct(p)) {
		printf("0\n");
		return 0;
	}
	printf("1\n");
	for (auto& row : built)
		for (int j = 0; j < n; j++) printf("%d%c", row[j], j + 1 < n ? ' ' : '\n');
}
