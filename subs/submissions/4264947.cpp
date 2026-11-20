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

template <typename T>
using min_pq = priority_queue<T, vector<T>, greater<T>>;

void solve() {
	int n;
	cin >> n;
	vector<int> graph[n];
	int arr[n], deg[n] {};
	for (int i = 0; i < n; i++) {
		cin >> arr[i] >> deg[i];
		for (int j = 0; j < deg[i]; j++) {
			int v;
			cin >> v;
			v--;
			graph[v].push_back(i);
		}
	}
	int ans = 0;
	min_pq<pair<int, int>> pq;
	auto push = [&](int u, int d) -> void {
		if (!(--deg[u])) {
			pq.emplace(d, u);
		}
	};
	for (int i = 0; i < n; i++) {
		if (!deg[i]) {
			pq.emplace(0, i);
		}
	}
	while (sz(pq)) {
		auto [d, u] = pq.top();
		dbg(u);
		ans = d + arr[u];
		pq.pop();
		for (auto& v : graph[u]) {
			push(v, d + arr[u]);
		}
	}
	cout << ans << endl;
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