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
	int n, m;
	cin >> n >> m;
	bool arr[n];
	for (int i = 0; i < n; i++) {
		cin >> arr[i];
	}
	vector<pair<int, bool>> graph[n];
	for (int i = 0; i < m; i++) {
		int u, v;
		cin >> u >> v;
		u--;
		v--;
		graph[u].emplace_back(v, true);
		graph[v].emplace_back(u, false);
	}
	bool qd[n]{};
	vector<int> q;
	int state[n], dep[n]{};
	memset(state, -1, sizeof(state));

	auto push = [&](int u, bool f = false) -> void {
		if (!qd[u] && (f || !dep[u])) {
			qd[u] = true;
			q.push_back(u);
		}
	};

	for (int i = 0; i < n; i++) {
		for (auto& [_, t] : graph[i]) {
			dep[i] += t;
		}
	}
	for (int i = 0; i < n; i++) {
		push(i);
	}

	while (sz(q)) {
		int u = q.back();
		q.pop_back();

		bool ans;
		if (arr[u]) {
			ans = false;
			for (auto& [v, t] : graph[u]) {
				if (t && state[v] != -1) {
					ans |= state[v];
				}
			}
		} else {
			ans = true;
			for (auto& [v, t] : graph[u]) {
				if (t) {
					assert(state[v] != -1);
					ans &= state[v];
				}
			}
		}
		int sz = 0;
		for (auto& [_, t] : graph[u]) {
			sz += t;
		}
		if (!sz) {
			ans = true;
		}
		dbg(u, sz, ans);

		state[u] = ans;
		for (auto& [v, t] : graph[u]) {
			if (!t) {
				dep[v]--;
				push(v, arr[v] && ans);
			}
		}
	}

	for (int i = 0; i < n; i++) {
		if (state[i] == -1) {
			state[i] = 0;
		}
		if (i) {
			cout << " ";
		}
		cout << state[i];
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