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

void solve() {
	int n, m, q;
	cin >> n >> m >> q;
	vector<int> graph[n];
	for (int i = 0; i < m; i++) {
		int u, v;
		cin >> u >> v;
		u--;
		v--;
		graph[v].push_back(u);
	}
	int arr[n];
	for (auto& a : arr) {
		cin >> a;
		a--;
	}
	int dist[n];
	memset(dist, 0x3f, sizeof(dist));
	queue<pair<int, int>> cq;
	auto push = [&](int u, int d) -> void {
		if (d < dist[u]) {
			dist[u] = d;
			cq.emplace(u, d);
		}
	};
	push(n - 1, 0);
	while (sz(cq)) {
		auto [u, d] = cq.front();
		cq.pop();
		for (auto& v : graph[u]) {
			push(v, d + 1);
		}
	}
	multiset<int> ans;
	for (int i = 0; i < n; i++) {
		ans.insert(i + dist[arr[i]]);
	}
	while (q--) {
		int u, v;
		cin >> u >> v;
		u--;
		v--;
		ans.erase(ans.find(u + dist[arr[u]]));
		ans.erase(ans.find(v + dist[arr[v]]));
		swap(arr[u], arr[v]);
		ans.insert(u + dist[arr[u]]);
		ans.insert(v + dist[arr[v]]);
		cout << *ans.begin() << endl;
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