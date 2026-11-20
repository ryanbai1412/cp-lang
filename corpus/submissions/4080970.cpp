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
	int n;
	cin >> n;
	bool used[n] {};
	int arr[n];
	for (auto& a : arr) {
		cin >> a;
		a--;
	}
	int inv[n];
	for (int i = 0; i < n; i++) {
		inv[arr[i]] = i;
	}
	auto lis = [&]() -> vector<int> {
		int g[n + 1], from[n];
		memset(g, 0x3f, sizeof(g));
		g[0] = -1e9;
		for (int i = 0; i < n; i++) {
			if (!used[i]) {
				int cur = lower_bound(g, g + n, arr[i]) - g;
				if (arr[i] < g[cur]) {
					g[cur] = arr[i];
					from[arr[i]] = g[cur - 1];
				} else {
					from[arr[i]] = -1;
				}
			}
		}
		vector<int> ans;
		int cur = 0;
		for (auto& a : g) {
			if (a < n) {
				cur = a;
			}
		}
		while (cur >= 0) {
			dbg(cur);
			used[inv[cur]] = true;
			ans.push_back(inv[cur]);
			cur = from[cur];
		}
		reverse(begin(ans), end(ans));
		return ans;
	};
	vector<vector<int>> ans;
	int mlis = -1;
	vector<int> last;
	while (sz(last = lis()) == mlis || mlis == -1) {
		mlis = sz(last);
		ans.push_back(last);
	}
	cout << sz(ans) << " " << mlis << endl;
	for (auto& a : ans) {
		for (int i = 0; i < sz(a); i++) {
			cout << a[i] + 1 << " \n"[i == sz(a) - 1];
		}
	}
}

int main() {
	cin.tie(nullptr);
	ios_base::sync_with_stdio(false);
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