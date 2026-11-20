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

const int maxn = 205;

struct ST {
	int n;
	vector<long> v, lazy;

	ST(int n) : n(n), v(4 * n, 4e18), lazy(4 * n) {}

	void maintain(int o) { v[o] = min(v[o * 2], v[o * 2 + 1]) + lazy[o]; }

	void update(int o, int l, int r, int ql, int qr, int x) {
		if (ql <= l && r <= qr) {
			lazy[o] += x;
			v[o] += x;
		} else {
			int mid = (l + r) / 2, lc = o * 2, rc = lc + 1;
			if (ql <= mid) {
				update(lc, l, mid, ql, qr, x);
			}
			if (mid < qr) {
				update(rc, mid + 1, r, ql, qr, x);
			}
			maintain(o);
		}
	}

	void update(int l, int r, int x) { update(1, 0, n - 1, l, r, x); }

	long query(int o, int l, int r, int ql, int qr) {
		if (ql <= l && r <= qr) {
			return v[o];
		} else {
			int mid = (l + r) / 2, lc = o * 2, rc = lc + 1;
			long ans = 4e18;
			if (ql <= mid) {
				ans = min(ans, query(lc, l, mid, ql, qr));
			}
			if (mid < qr) {
				ans = min(ans, query(rc, mid + 1, r, ql, qr));
			}
			return ans + lazy[o];
		}
	}

	long query(int l, int r) { return query(1, 0, n - 1, l, r); }
	
	void set(int ind, int x) { update(ind, ind, x - query(ind, ind)); }
};

void solve() {
	int n, l;
	cin >> n >> l;
	vector<int> arr(n);
	for (auto& a : arr) {
		cin >> a;
	}
	if (n <= 200) {
		int dp[n + 1][maxn + 1][maxn + 1] {};
		for (int i = n - 1; i >= 0; i--) {
			for (int j = 0; j <= maxn; j++) {
				for (int k = 0; k <= maxn; k++) {
					dp[i][j][k] = 1e9;
					for (int a = max(0, arr[i] - l); a <= arr[i]; a++) {
						dp[i][j][k] =
							min({dp[i][j][k], dp[i + 1][a][k] + abs(j - a),
								 dp[i + 1][j][a] + abs(k - a)});
					}
				}
			}
		}
		cout << dp[0][0][0] << endl;
	} else {
		n++;
		arr.insert(arr.begin(), 0);
//		long dp[n][n];
//		memset(dp, 0x3f, sizeof(dp));
//		for (int i = 0; i < n; i++) {
//			for (int j = 0; j < i - 1; j++) {
//				dp[i][j] = abs(arr[i] - arr[i - 1]) + dp[i - 1][j];
//			}
//			for (int k = 0; k < i; k++) {
//				dp[i][i - 1] = min(dp[i][i - 1], dp[i - 1][k] + abs(arr[i] - arr[k]));
//			}
//		}
//		cout << *min_element(dp[n - 1], dp[n - 1] + n) << endl;
		map<int, int> order;
		for (auto& a : arr) {
			order[a] = 0;
		}
		int ind = 0;
		for (auto& [k, v] : order) {
			v = ind++;
		}
		ST dpneg(ind), dppos(ind);
		for (int i = 1; i < n; i++) {
			dpneg.update(i, i - 2, abs(arr[i] - arr[i - 1]));
			dppos.update(i, i - 2, abs(arr[i] - arr[i - 1]));
			int cind = order[arr[i]];
			long dp = min(arr[i] + dpneg.query(0, cind), dppos.query(cind , ind - 1) - arr[i]);
			dpneg.set(cind, dp - arr[i]);
			dppos.set(cind, dp + arr[i]);
		}
		long ans = 4e18;
		for (int i = 0; i < n; i++) {
			ans = min(ans, dpneg.query(i, i) + arr[i]);
		}
		cout << ans << endl;
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