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

const int inf = 1e9;

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
	int mod = 2 * k + 1;
	auto cdiv = [&](int a, int b) -> int { return (a + b - 1) / b; };
	auto compute = [&](int x) -> int {
		if (x < 0) {
			return 0;
		}
		return cdiv(x, mod);
	};
	int n = sz(arr);
	int dp[3] {0, inf, inf};
	int prev = 0;
	for (auto& [x, cnt] : arr) {
		int ndp[3] {inf, inf, inf};
		auto process = [&](int dpv, int cnt) -> void {
			for (int i = 0; i < 3; i++) {
				int xcnt = cnt;
				if (i == 1) {
					xcnt++;
				} else if (i == 2) {
					xcnt -= k + 1;
				}
				ndp[i] = min(ndp[i], dpv + compute(xcnt));
			}
		};
		process(dp[0], cnt);
		if (prev > x) {
			process(dp[1], cnt - k - 1);
		} else {
			process(dp[2], cnt + 1);
		}
		prev = x;
		memcpy(dp, ndp, sizeof(dp));
	}
	cout << dp[0] << endl;
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