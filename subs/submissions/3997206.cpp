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

const int maxn = 5005;
const long mod = 998244353;

long choose[maxn][maxn + 1];

void pcomp() {
	for (int i = 0; i < maxn; i++) {
		choose[i][0] = choose[i][i] = 1;
		for (int j = 1; j < i; j++) {
			choose[i][j] = choose[i - 1][j - 1] + choose[i - 1][j];
			if (choose[i][j] >= mod) {
				choose[i][j] -= mod;
			}
		}
	}
}

vector<tuple<long, long, long>> solve(vector<string> xarr) {
	int n = sz(xarr);
	vector<int> arr(n);
	for (int i = 0; i < n; i++) {
		arr[i] = count(begin(xarr[i]), end(xarr[i]),'1');
	}
	sort(begin(arr), end(arr), greater<>());
	vector<int> psum(n + 1);
	partial_sum(begin(arr), end(arr), psum.begin() + 1);
	auto sum = [&](int l, int r) -> int {
		return psum[r + 1] - psum[l];
	};
	vector<tuple<long, long, long>> ans;
	for (int i = 0; i <= n; i++) {
		long mx = sum(0, i - 1);
		mx += n * (n - i);
		mx -= sum(i, n - 1);
		long ways = 1;
		if (i && i < n) {
			int cval = arr[i - 1];
			int lrange = lower_bound(begin(arr), end(arr), cval, greater<>()) - arr.begin();
			int range = upper_bound(begin(arr), end(arr), cval, greater<>()) - arr.begin();
			ways = choose[range - lrange][i - lrange];
		}
		ans.emplace_back(mx, ways, i);
	}
	return ans;
}

void solve() {
	int n;
	cin >> n;
	vector<string> arr(n), iarr(n, string(n, ' '));
	for (auto& a : arr) {
		cin >> a;
	}
	for (int i = 0; i < n; i++) {
		for (int j = 0; j < n; j++) {
			iarr[i][j] = arr[j][i];
		}
	}
	auto ansa = solve(arr), ansb = solve(iarr);
	long ans = 0;
	for (auto& [mxa, wa, xa] : ansa) {
		for (auto& [mxb, wb, xb] : ansb) {
			long csum = mxa + mxb - xa * xb - (n - xa) * (n - xb);
			if (csum == n * n) {
				ans = (ans + wa * wb) % mod;
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
	pcomp();
	int t = 1;
	// cin >> t;
	for (int _ = 1; _ <= t; _++) {
		dbg(_);
		// cout << "Case #" << _ << ": ";
		solve();
	}
}