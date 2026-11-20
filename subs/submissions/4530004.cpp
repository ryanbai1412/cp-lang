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

void solve() {
	int n, m;
	cin >> n >> m;
	vector<int> graph[n], igraph[n];
	for (int i = 0; i < m; i++) {
		int u, v;
		cin >> u >> v;
		u--;
		v--;
		graph[u].push_back(v);
		igraph[v].push_back(u);
	}
	int in[n], out[n];
	for (int i = 0; i < n; i++) {
		in[i] = sz(igraph[i]);
		out[i] = sz(graph[i]);
	}
	bool vis[n] {};
	vector<int> q;
	vector<pair<int, int>> ans;
	auto push = [&](int u) -> void {
		if (vis[u]) {
			return;
		}
		if (!in[u] || !out[u]) {
			vis[u] = true;
			q.push_back(u);
		}
	};
	for (int i = 0; i < n; i++) {
		push(i);
	}
	while (sz(q)) {
		int u = q.back();
		q.pop_back();
		if (in[u]) {
			ans.emplace_back(u, 0);
		} else if (out[u]) {
			ans.emplace_back(u, 1);
		} else {
			ans.emplace_back(u, 0);
		}
		for (auto& a : graph[u]) {
			in[a]--;
			push(a);
		}
		for (auto& a : igraph[u]) {
			out[a]--;
			push(a);
		}
	}
	for (auto& a : vis) {
		if (!a) {
			cout << -1 << endl;
			return;
		}
	}
	cout << sz(ans) << endl;
	for (auto& [x, y] : ans) {
		cout << x + 1 << " " << y << endl;
	}
}

int main() {
	cin.tie(nullptr);
	ios_base::sync_with_stdio(false);
	cin.exceptions(ios::failbit);
	solve();
}