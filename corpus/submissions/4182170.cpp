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

struct BDS {
	map<long, long> m;
	
	void insert(long k, long v) {
		auto it = m.upper_bound(k);
		if (it != m.begin() && prev(it)->second <= v) {
			return;
		}
		while (it != m.end() && it->second > v) {
			it = m.erase(it);
		}
		m[k] = v;
	}
	
	long qmin(long x) const {
		auto it = m.upper_bound(x);
		if (it == m.begin()) {
			return 1e18;
		}
		return (--it)->second;
	}
};

void solve() {
	int n;
	cin >> n;
	long arr[n + 1][2];
	for (int i = 1; i <= n; i++) {
		cin >> arr[i][0] >> arr[i][1];
	}
	arr[0][0] = arr[0][1] = 0;
	long psums[n + 1][2];
	psums[0][0] = psums[0][1] = 0;
	for (int i = 0; i < n; i++) {
		psums[i + 1][0] = psums[i][0] + abs(arr[i][0] - arr[i + 1][0]);
		psums[i + 1][1] = psums[i][1] + abs(arr[i][1] - arr[i + 1][1]);
	}
	auto sum = [&](int t, int l, int r) -> long {
		return psums[r][t] - psums[l][t];
	};
	BDS dp1s, dp1g, dp2s, dp2g;
	long dp1[n + 1], dp2[n + 1];
	dp1[n] = dp2[n] = 0;
	auto upd1 = [&](int i) -> void {
		if (i) {
			long base = psums[i - 1][0] + dp1[i];
			dp1s.insert(arr[i][1], base - arr[i][1]);
			dp1g.insert(-arr[i][1], base + arr[i][1]);
		}
	};
	auto upd2 = [&](int i) -> void {
		if (i) {
			long base = psums[i - 1][1] + dp2[i];
			dp2s.insert(arr[i][0], base - arr[i][0]);
			dp2g.insert(-arr[i][0], base + arr[i][0]);
		}
	};
	upd1(n);
	upd2(n);
	for (int i = n - 1; i >= 0; i--) {
		dp1[i] = dp2[i] = 1e18;
		{
			long pcx = i ? arr[i - 1][0] : 0;
			dp1[i] = min(dp2s.qmin(pcx) + pcx, dp2g.qmin(-pcx) - pcx);
			dp1[i] = min(dp1[i] - psums[i][1], sum(1, i, n));
		}
		{
			long pcx = i ? arr[i - 1][1] : 0;
			dp2[i] = min(dp1s.qmin(pcx) + pcx, dp1g.qmin(-pcx) - pcx);
			dp2[i] = min(dp2[i] - psums[i][0], sum(0, i, n));
		}
		upd1(i);
		upd2(i);
		dbg(i, dp1[i], dp2[i]);
	}
	cout << min(dp1[0], dp2[0]) << endl;
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