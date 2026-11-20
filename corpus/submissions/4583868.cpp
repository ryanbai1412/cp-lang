#include "bits/extc++.h"

using namespace std;

template <typename T>
void dbgh(const T&t) {
	cerr << t << endl;
}

template <typename T, typename...U>
void dbgh(const T&t, const U&...u) {
	cerr << t << " | ";
	dbgh(u...);
}

#ifdef DEBUG
#define dbg(...) cerr << "L" << __LINE__ << " [" << #__VA_ARGS__ << "]: "; dbgh(__VA_ARGS__);
#else
#define dbg(...)
#define cerr if (false) cerr
#endif

#define endl "\n"
#define long int64_t
#define sz(x) int((x).size())

#define int long

const int maxn = 1e6 + 5;

vector<int> cv, graph[maxn];
bool vis[maxn];

void dfs(int u) {
	if (vis[u]) {
		return;
	}
	vis[u] = true;
	cv.push_back(u);
	for (auto& v : graph[u]) {
		dfs(v);
	}
}

void solve() {
	int n, m, k;
	cin >> n >> m >> k;
	for (int i = 0; i < m; i++) {
		int u, v;
		cin >> u >> v;
		u--;
		v--;
		graph[u].push_back(v);
		graph[v].push_back(u);
	}
	int ans = 0, bad = 0;
	for (int i = 0; i < n; i++) {
		if (!vis[i]) {
			cv.clear();
			dfs(i);
			bool cyc = true;
			for (auto& a : cv) {
				cyc &= sz(graph[a]) == 2;
			}
			dbg(sz(cv), cyc);
			int x = sz(cv) / k;
			ans += x;
			if (!cyc) {
				if (sz(cv) % k == 0) {
					bad += max(0, x - 1);
				} else {
					bad += x;
				}
			} else {
				if (sz(cv) % k == 0) {
					bad += x;
				} else {
					bad += x + 1;
				}
			}
		}
	}
	cout << ans * k << " " << bad << endl;
}

int32_t main() {
	cin.tie(nullptr);
	cin.exceptions(ios::failbit);
	ios_base::sync_with_stdio(false);
	solve();
}