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
	int k;
	vector<pair<int, int>> arr;
	{
		int n;
		cin >> n >> k;
		int carr[n];
		for (auto& a : carr) {
			cin >> a;
		}
		for (int i = 0; i < n;) {
			int start = carr[i], sind = i;
			for (; i < n && carr[i] == start; i++)
				;
			arr.emplace_back(start, i - sind);
		}
	}
	int n = sz(arr);
	int mod = 2 * k + 1;
	auto get = [&](int ind) -> int {
		if (0 <= ind && ind < n) {
			return arr[ind].first;
		}
		return -1;
	};
	auto cdiv = [&](int a, int b) -> int { return (a + b - 1) / b; };
	auto compute = [&](int x) -> int { return cdiv(x, mod); };
	int ans = 0;
	bool help[n]{};
	for (int i = 0; i < n; i++) {
		int cur = arr[i].second % mod;
		help[i] = 0 < cur && cur <= k + 1;
		dbg(i, help[i]);
	}
	for (int i = 0; i < n; i++) {
		ans += compute(arr[i].second);
		int ctype = arr[i].second % mod;
		if (ctype) {
			ctype = mod - ctype;
		}
		dbg(i, ctype, arr[i].second, compute(arr[i].second), ans);
		if (ctype == 1) {
			if (get(i) > get(i - 1) && i && help[i - 1]) {
				ans -= help[i - 1];
				help[i - 1] = false;
			} else if (get(i) > get(i + 1) && i + 1 < n) {
				ans -= help[i + 1];
				help[i + 1] = false;
			}
		} else if (ctype >= 2) {
			if (get(i) > get(i - 1) && i) {
				ans -= help[i - 1];
				help[i - 1] = false;
			}
			if (get(i) > get(i + 1) && i + 1 < n) {
				ans -= help[i + 1];
				help[i + 1] = false;
			}
		}
	}
	cout << ans << endl;
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