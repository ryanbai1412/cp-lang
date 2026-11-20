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

using G = vector<vector<int>>;

bool solve1(const G &g) {
	return false;
}

bool solve2(const G &g) {
	return false;
}

bool solve3(const G &g) {
	return false;
}

bool solve4(const G &g) {
	int mx = 0;
	for (auto& a : g) {
		mx = max(mx, sz(a));
	}
	return mx >= 3;
}

bool solve5(const G &g) {
	return false;
}

void solve() {
	int s;
	cin >> s;
	int tcs;
	cin >> tcs;
	while (tcs--) {
		int n, m;
		cin >> n >> m;
		G graph(n);
		for (int i = 0; i < m; i++) {
			int u, v;
			cin >> u >> v;
			u--;
			v--;
			graph[u].push_back(v);
			graph[v].push_back(u);
		}
		bool ans;
		if (s == 1) {
			ans = solve1(graph);
		} else if (s == 2) {
			ans = solve2(graph);
		} else if (s == 3) {
			ans = solve3(graph);
		} else if (s == 4) {
			ans = solve4(graph);
		} else if (s == 5) {
			ans = solve5(graph);
		} else {
			assert(false);
		}
		if (ans) {
			cout << "YES" << endl;
		} else {
			cout << "NO" << endl;
		}
	}
}

int main() {
	cin.tie(nullptr);
	cin.exceptions(ios::failbit);
	ios_base::sync_with_stdio(false);
	solve();
}