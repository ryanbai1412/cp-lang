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
#define dbg(...) 1412
#endif

#define endl "\n"
#define long int64_t
#define sz(x) int((x).size())

const int maxn = 1e5 + 5;

bool vis[maxn];
int t, cs, comp[maxn], tin[maxn], tout[maxn], low[maxn];
vector<int> graph[maxn], tree[maxn];

void dfs(int u, int p = -1) {
	low[u] = tin[u] = t++;
	vis[u] = true;
	comp[u] = cs;
	for (auto& v : graph[u]) {
		if (vis[v]) {
			if (v != p) {
				dbg("back", u, v);
				low[u] = min(low[u], tin[v]);
			}
		} else {
			dbg("tree", u, v);
			tree[u].push_back(v);
			dfs(v, u);
			low[u] = min(low[u], low[v]);
		}
	}
	tout[u] = t++;
}

bool anc(int u, int v) {
	return tin[u] <= tin[v] && tout[v] <= tout[u];
}

int imm(int u, int v) {
	assert(u != v);
	if (!anc(u, v)) {
		return -1;
	}
	int l = 0, r = sz(tree[u]) - 1;
	while (l < r) {
		int mid = (l + r + 1) / 2;
		if (tin[tree[u][mid]] <= tin[v]) {
			l = mid;
		} else {
			r = mid - 1;
		}
	}
	return tree[u][l];
}

void solve() {
	int n, m;
	cin >> n >> m;
	for (int i = 0; i < m; i++) {
		int u, v;
		cin >> u >> v;
		u--;
		v--;
		graph[u].push_back(v);
		graph[v].push_back(u);
	}
	for (int i = 0; i < n; i++) {
		if (!vis[i]) {
			dfs(i);
			cs++;
		}
	}
	int q;
	cin >> q;
	while (q--) {
		dbg("iter");
		int ty, a, b;
		cin >> ty >> a >> b;
		a--;
		b--;
		if (ty == 1) {
			int u, v;
			cin >> u >> v;
			if (comp[a] != comp[b]) {
				cout << "no" << endl;
				continue;
			}
			u--;
			v--;
			if (!anc(u, v)) {
				swap(u, v);
			}
			if (low[v] <= tin[u] || anc(v, a) == anc(v, b)) {
				cout << "yes" << endl;
			} else {
				cout << "no" << endl;
			}
		} else {
			assert(ty == 2);
			int u;
			cin >> u;
			if (comp[a] != comp[b]) {
				cout << "no" << endl;
				continue;
			}
			u--;
			dbg(u, a, b);
			bool ua = anc(u, a), ub = anc(u, b);
			if (!ua && !ub) {
				cout << "yes" << endl;
			} else if (ua && ub) {
				int ia = imm(u, a), ib = imm(u, b);
				if (ia == ib || (low[ia] < tin[u] && low[ib] < tin[u])) {
					cout << "yes" << endl;
				} else {
					cout << "no" << endl;
				}
			} else {
				if (ub) {
					swap(a, b);
					swap(ua, ub);
				}
				if (low[imm(u, a)] < tin[u]) {
					cout << "yes" << endl;
				} else {
					cout << "no" << endl;
				}
			}
		}
	}
}

int main() {
	cin.tie(nullptr);
	ios_base::sync_with_stdio(false);
	cin.exceptions(ios::failbit);
	int t = 1;
	// cin >> t;
	for (int _ = 1; _ <= t; _++) {
		dbg(_);
		// cout << "Case #" << _ << ": ";
		solve();
	}
}