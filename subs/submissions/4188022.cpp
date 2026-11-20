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
	int n, m;
	cin >> n >> m;
	map<string, int> conv;
	int ind = 0;
	auto gind = [&](const string &s) -> int {
		auto it = conv.lower_bound(s);
		if (it == conv.end() || it->first != s) {
			conv.emplace_hint(it, s, ind);
			return ind++;
		}
		return it->second;
	};
	vector<pair<int, long>> graph[n];
	for (int i = 0; i < m; i++) {
		string u, v;
		long w;
		cin >> u >> v >> w;
		graph[gind(u)].emplace_back(gind(v), w);
	}
	int q;
	cin >> q;
	while (q--) {
		string a, b;
		cin >> a >> b;
		int s = gind(a), e = gind(b);
		long dist[n];
		memset(dist, 0x4f, sizeof(dist));
		min_pq<pair<long, int>> pq;
		auto push = [&](int u, long d) -> void {
			if (d < dist[u]) {
				dist[u] = d;
				pq.emplace(d, u);
			}
		};
		push(s, 0);
		while (sz(pq)) {
			auto [d, u] = pq.top();
			pq.pop();
			if (d == dist[u]) {
				for (auto& [v, w] : graph[u]) {
					push(v, d + w);
				}
			}
		}
		if (dist[e] < long(1e18)) {
			cout << dist[e] << endl;
		} else {
			cout << "Roger" << endl;
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