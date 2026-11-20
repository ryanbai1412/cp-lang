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

const int maxn = 8e5 + 5;

vector<int> graph[maxn];
vector<pair<int, int>> tour;

void dfs(int u, int p = -1, int t = 0) {
	for (auto& v : graph[u]) {
		if (v == p) {
			t++;
			continue;
		}
		tour.emplace_back(t, v);
		dfs(v, u, t);
		tour.emplace_back(t, u);
	}
}

void solve() {
	int n, q;
	cin >> n >> q;
	for (int i = 0; i < n; i++) {
		int m;
		cin >> m;
		graph[i].resize(m);
		for (auto& a : graph[i]) {
			cin >> a;
			a--;
		}
		graph[i].push_back(graph[i][0]);
		graph[i].erase(graph[i].begin());
	}
	dfs(0);
	int cnt[n + 1] {};
	for (auto& [a, _] : tour) {
		cnt[a]++;
	}
	long psum1[n + 1];
	for (int i = 0; i <= n; i++) {
		psum1[i] = (i ? psum1[i - 1] : 0) + cnt[i];
	}
	long psum[n + 1];
	for (int i = 0; i <= n; i++) {
		psum[i] = (i ? psum[i - 1] : 0) + psum1[i];
	}
	while (q--) {
		long x;
		cin >> x;
		int ind = int(lower_bound(psum, psum + n + 1, x) - psum);
		if (ind) {
			x -= psum[ind - 1];
		}
		if (ind > n) {
			x %= 2 * (n - 1);
		}
		if (!x) {
			cout << 1 << endl;
			continue;
		}
		for (auto& [a, b] : tour) {
			if (a <= ind) {
				x--;
				if (!x) {
					cout << b + 1 << endl;
					break;
				}
			}
		}
	}
}

int main() {
	cin.tie(nullptr);
	ios_base::sync_with_stdio(false);
	cin.exceptions(ios::failbit);
	solve();
}