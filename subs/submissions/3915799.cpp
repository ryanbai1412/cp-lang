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

const int maxn = 5e5;

bool arr[maxn];
int memo[maxn];
vector<int> graph[maxn];

int dp(int u) {
	int& ans = memo[u];
	if (ans != -1) {
		return ans;
	}
	if (find(begin(graph[u]), end(graph[u]), u) != graph[u].end()) {
		if (!arr[u]) {
			return (ans = false);
		}
	}
	if (!sz(graph[u])) {
		return (ans = true);
	}
	if (arr[u]) {
		ans = false;
		for (auto& v : graph[u]) {
			if (u != v) {
				ans |= dp(v);
			}
		}
	} else {
		ans = true;
		for (auto& v : graph[u]) {
			assert(u != v);
			ans &= dp(v);
		}
	}
	return ans;
}

void solve() {
	int n, m;
	cin >> n >> m;
	for (int i = 0; i < n; i++) {
		cin >> arr[i];
	}
	for (int i = 0; i < m; i++) {
		int u, v;
		cin >> u >> v;
		u--;
		v--;
		graph[u].push_back(v);
		assert(u <= v);
	}
	for (int i = 0; i < n; i++) {
		if (i) {
			cout << " ";
		}
		cout << dp(i);
	}
	cout << endl;
}

int main() {
	cin.tie(0)->sync_with_stdio(0);
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