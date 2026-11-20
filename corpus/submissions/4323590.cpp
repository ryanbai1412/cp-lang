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

const int maxn = 1e6 + 5;

vector<int> graph[maxn];
bool vis[maxn];

void solve() {
	int n, m;
	cin >> n >> m;
	int arr[n][m];
	for (auto& a : arr) {
		for (auto& b : a) {
			cin >> b;
		}
	}
	for (int i = 0; i < n; i++) {
		for (int j = 0; j < m; j++) {
			graph[(i + 1) * (j + 1)].push_back(arr[i][j]);
		}
	}
	vector<int> q;
	auto push = [&](int u) -> void {
		if (!vis[u]) {
			vis[u] = true;
			q.push_back(u);
		}
	};
	push(arr[0][0]);
	while (sz(q)) {
		int u = q.back();
		q.pop_back();
		for (auto& v : graph[u]) {
			push(v);
		}
	}
	if (vis[n * m]) {
		cout << "yes" << endl;
	} else {
		cout << "no" << endl;
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