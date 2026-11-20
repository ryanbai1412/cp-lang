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

struct DSU {
	vector<int> p;
	
	DSU(int n): p(n, -1) {}
	
	int find(int u) {
		return p[u] < 0 ? u : (p[u] = find(p[u]));
	}
	
	bool merge(int u, int v) {
		u = find(u);
		v = find(v);
		if (u == v) {
			return false;
		}
		if (p[u] < p[v]) {
			swap(u, v);
		}
		p[v] += p[u];
		p[u] = v;
		return true;
	}
};

void solve() {
	int n, m;
	cin >> n >> m;
	vector<int> groups[n / m];
	for (int i = 0; i < n / m; i++) {
		groups[i].resize(m);
		for (auto& a : groups[i]) {
			cin >> a;
			a--;
		}
	}
	int points[n] {};
	for (int i = 0; i < (m - 1) * n / 2; i++) {
		int u, v;
		char res;
		cin >> u >> v >> res;
		u--;
		v--;
		if (res == 'W') {
			points[u] += 3;
		} else if (res == 'T') {
			points[u]++;
			points[v]++;
		} else {
			points[v] += 3;
		}
	}
	int k;
	cin >> k;
	k--;
	bool first = true;
	for (auto& group : groups) {
		sort(begin(group), end(group), [&](int a, int b) -> bool {
			if (points[a] == points[b]) {
				return a < b;
			}
			return points[a] > points[b];
		});
//		dbg(group[0], group[1]);
		if (!first) {
			cout << " ";
		}
		first = false;
		cout << group[k] + 1;
	}
	cout << endl;
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