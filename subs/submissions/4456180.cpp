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
#define sz(x) int((x).size())

const int maxn = 1e5 + 5, logn = 20;

int t, tin[maxn], tout[maxn], lift[logn][maxn];
vector<int> graph[maxn];

void dfs(int u, int p = -1) {
	tin[u] = t++;
	lift[0][u] = p;
	for (auto& v : graph[u]) {
		if (v != p) {
			dfs(v, u);
		}
	}
	tout[u] = t++;
}

bool anc(int u, int v) {
	return tin[u] <= tin[v] && tout[v] <= tout[u];
}

int lca(int u, int v) {
	if (anc(u, v)) {
		return u;
	}
	for (int i = logn - 1; i >= 0; i--) {
		int nu = lift[i][u];
		if (nu != -1 && !anc(nu, v)) {
			u = nu;
		}
	}
	return lift[0][u];
}

void solve() {
	int n, q, k;
	cin >> n >> q >> k;
	int edges[n - 1][2];
	for (auto& [u, v] : edges) {
		cin >> u >> v;
		u--;
		v--;
		graph[u].push_back(v);
		graph[v].push_back(u);
	}
	dfs(0);
	for (int i = 1; i < logn; i++) {
		for (int j = 0; j < n; j++) {
			if (lift[i - 1][j] == -1) {
				lift[i][j] = -1;
			} else {
				lift[i][j] = lift[i - 1][lift[i - 1][j]];
			}
		}
	}
	int psum[n * 2 + 1] {};
	auto add = [&](int l, int r) -> void {
		psum[l]++;
		psum[r + 1]--;
	};
	while (q--) {
		dbg("Q");
		int m;
		cin >> m;
		int arr[m];
		for (auto& a : arr) {
			cin >> a;
			a--;
		}
		sort(arr, arr + m, [&](int a, int b) -> bool {
			return tin[a] < tin[b];
		});
		vector<int> vtree(arr, arr + m);
		for (int i = 0; i < m - 1; i++) {
			vtree.push_back(lca(arr[i], arr[i + 1]));
		}
		sort(begin(vtree), end(vtree), [&](int a, int b) -> bool {
			return tin[a] < tin[b];
		});
		vtree.erase(unique(begin(vtree), end(vtree)), vtree.end());
		vector<int> stk;
		for (auto& a : vtree) {
			dbg(a);
		}
		for (auto& a : vtree) {
			while (sz(stk) && !anc(stk.back(), a)) {
				stk.pop_back();
			}
			if (sz(stk)) {
				dbg(stk.back(), a);
				add(tin[stk.back()] + 1, tin[a]);
			}
			stk.push_back(a);
		}
	}
	for (int i = 0; i < 2 * n; i++) {
		psum[i + 1] += psum[i];
	}
	for (int i = 0; i < n; i++) {
		dbg(i, psum[tin[i]] - psum[tout[i]]);
	}
	vector<int> ans;
	for (int i = 0; i < n - 1; i++) {
		auto [u, v] = edges[i];
		if (lift[0][u] == v) {
			swap(u, v);
		}
		if (psum[tin[v]] - psum[tout[v]] >= k) {
			ans.push_back(i);
		}
	}
	cout << sz(ans) << endl;
	for (int i = 0; i < sz(ans); i++) {
		cout << ans[i] + 1 << " \n"[i == sz(ans) - 1];
	}
}

int main() {
	cin.tie(nullptr);
	ios_base::sync_with_stdio(false);
	cin.exceptions(ios::failbit);
	solve();
}