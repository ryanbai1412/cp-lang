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

vector<bool> solve(int n,
				   const vector<pair<int, int>>& edges,
				   const vector<bool>& losing) {
	int deg[n] {};
	vector<int> graph[n];
	for (auto& [u, v] : edges) {
		deg[u]++;
		graph[v].push_back(u);
	}
	vector<int> q;
	vector<bool> ans(n, true);
	auto push = [&](int u) -> void {
		if (ans[u]) {
			ans[u] = false;
			q.push_back(u);
		}
	};
	for (int i = 0; i < n; i++) {
		if (losing[i]) {
			push(i);
		}
	}
	while (sz(q)) {
		int u = q.back();
		q.pop_back();
		for (auto& v : graph[u]) {
			if (v % 2 == 1) {
				push(v);
			} else if (!(--deg[v])) {
				push(v);
			}
		}
	}
	return ans;
}

void solve() {
	int n, m;
	cin >> n >> m;
	vector<int> grapha[n], graphb[n];
	for (int i = 0; i < n - 1; i++) {
		int u, v;
		cin >> u >> v;
		u--;
		v--;
		grapha[u].push_back(v);
		grapha[v].push_back(u);
		graphb[u].push_back(v);
		graphb[v].push_back(u);
	}
	for (int i = 0; i < m; i++) {
		int u, v;
		cin >> u >> v;
		u--;
		v--;
		graphb[u].push_back(v);
		graphb[v].push_back(u);
	}
	int cn = 2 * n * n;
	auto comp = [&](int i, int j, bool turn) -> int {
		return ((i * n + j) << 1) | turn;
	};
	auto decomp = [&](int x) -> tuple<int, int, bool> {
		bool turn = x & 1;
		x >>= 1;
		return {x / n, x % n, turn};
	};
	int deg[cn] {};
	for (int i = 0; i < n; i++) {
		for (int j = 0; j < n; j++) {
			deg[comp(i, j, false)] = sz(graphb[i]) + 1;
		}
	}
	bool ans[cn] {};
	memset(ans, 1, sizeof(ans));
	vector<int> q;
	auto push = [&](int x) -> void {
		if (ans[x]) {
			ans[x] = false;
			q.push_back(x);
		}
	};
	for (int i = 0; i < n; i++) {
		push(comp(i, i, false));
		push(comp(i, i, true));
	}
	dbg(deg[comp(1, 0, false)]);
	while (sz(q)) {
		auto [u, v, turn] = decomp(q.back());
		q.pop_back();
		if (turn) {
			dbg(u, v);
			for (auto& x : graphb[u]) {
				int nxt = comp(x, v, false);
				dbg(x, v, "from", u, v);
				if (!(--deg[nxt])) {
					push(nxt);
				}
			}
			int nxt = comp(u, v, false);
			if (!(--deg[nxt])) {
				push(nxt);
			}
		} else {
			for (auto& x : grapha[v]) {
				push(comp(u, x, true));
			}
			push(comp(u, v, true));
		}
	}
	int cans = 0;
	for (int i = 0; i < n; i++) {
		for (int j = 0; j < n; j++) {
			if (i != j && ans[comp(i, j, false)]) {
				dbg(i, j);
				cans++;
			}
		}
	}
	cout << cans << endl;
}

int main() {
	cin.tie(nullptr);
	ios_base::sync_with_stdio(false);
	cin.exceptions(ios::failbit);
#ifdef LOCAL
	freopen("input.txt", "r", stdin);
#endif
	int t = 1;
	// cin >> t;
	for (int _ = 1; _ <= t; _++) {
		dbg(_);
		// cout << "Case #" << _ << ": ";
		solve();
	}
}