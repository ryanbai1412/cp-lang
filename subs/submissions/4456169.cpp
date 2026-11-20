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

int solver(const vector<bool> &arr) {
	int n = sz(arr);
	int dp[n + 1];
	dp[n] = 0;
	for (int i = n - 1; i >= 0; i--) {
		dp[i] = 1 + dp[i + 1];
		if (i + 1 < n) {
			dp[i] = min(dp[i], 1 + arr[i] + arr[i + 1] + dp[i + 2]);
		}
	}
	return dp[0];
}

int solve(vector<bool> arr) {
	if (sz(arr) == 2) {
		return arr[0] + arr[1];
	}
	int ans = solver(arr);
	dbg(sz(arr));
	rotate(arr.begin(), arr.begin() + 1, arr.end());
	return min(ans, solver(arr));
}

void solve() {
	dbg(solve({0, 1, 0}));
	map<string, int> m;
	auto conv = [&](const string &s) -> int {
		static int sind = 0;
		auto [it, inserted] = m.emplace(s, 0);
		if (inserted) {
			return it->second = sind++;
		}
		return it->second;
	};
	int n;
	cin >> n;
	if (n & 1) {
		cout << -1 << endl;
		return;
	}
	int arr[n], edges[n][2];
	for (auto& [u, v] : edges) {
		string s, t;
		cin >> s >> t;
		u = conv(s);
		v = conv(t);
		arr[u] = v;
	}
	bool used[n] {}, cycle[n] {};
	vector<vector<int>> cycles;
	{
		int vis[n] {};
		for (int i = 0; i < n; i++) {
			if (vis[i]) {
				continue;
			} else {
				int j = i;
				vector<int> st;
				while (true) {
					if (vis[j] == 2) {
						break;
					} else if (vis[j] == 1) {
						vector<int> cur(find(begin(st), end(st), j), st.end());
						cycles.push_back(cur);
						for (auto& a : cur) {
							cycle[a] = true;
						}
						break;
					}
					st.push_back(j);
					vis[j] = 1;
					j = arr[j];
				}
				for (auto& a : st) {
					vis[a] = 2;
				}
			}
		}
	}
	int ideg[n] {};
	for (auto& a : arr) {
		ideg[a]++;
	}
	vector<int> q;
	for (int i = 0; i < n; i++) {
		if (!ideg[i]) {
			q.push_back(i);
		}
	}
	int ans = 0;
	while (sz(q)) {
		int u = q.back();
		q.pop_back();
		if (!used[u] && !used[arr[u]] && !cycle[arr[u]] && arr[u] != u) {
			ans++;
			used[u] = used[arr[u]] = true;
			int v = arr[arr[u]];
			if (!(--ideg[v])) {
				q.push_back(v);
			}
		}
	}
	memset(ideg, 0, sizeof(ideg));
	for (int i = 0; i < n; i++) {
		if (!used[i]) {
			ideg[arr[i]]++;
		}
	}
	dbg(ans);
	for (auto& a : cycles) {
		for (auto& b : a) {
			assert(!used[b]);
		}
		dbg(sz(a));
		vector<bool> cur;
		for (auto& b : a) {
			cur.push_back(ideg[b] > 1);
			dbg(b, ideg[b], cur.back());
			ans += max(0, ideg[b] - 2);
		}
		ans += solve(cur);
	}
	for (int i = 0; i < n; i++) {
		if (!used[i] && !cycle[i] && !cycle[arr[i]]) {
			ans++;
		}
	}
	cout << ans << endl;
}

int main() {
	cin.tie(nullptr);
	ios_base::sync_with_stdio(false);
	cin.exceptions(ios::failbit);
	solve();
}