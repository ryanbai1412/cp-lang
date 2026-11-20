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

struct custom_hash {
	static uint64_t splitmix64(uint64_t x) {
		// http://xorshift.di.unimi.it/splitmix64.c
		x += 0x9e3779b97f4a7c15;
		x = (x ^ (x >> 30)) * 0xbf58476d1ce4e5b9;
		x = (x ^ (x >> 27)) * 0x94d049bb133111eb;
		return x ^ (x >> 31);
	}

	size_t operator()(uint64_t x) const {
		static const uint64_t FIXED_RANDOM =
			chrono::steady_clock::now().time_since_epoch().count();
		return splitmix64(x + FIXED_RANDOM);
	}
};

const int maxn = 1e4 + 5;

int n, arr[maxn], psum[maxn];
__gnu_pbds::gp_hash_table<long, long, custom_hash> memo;

long sq(long x) {
	return x * x;
}

long encode(long x) {
	if (x < 0) {
		return x;
	} else {
		return x + 1;
	}
}

long decode(long x) {
	if (x < 0) {
		return x;
	} else {
		return x - 1;
	}
}

long dp(int ind, int sum) {
	if (ind == n - 1) {
		return -long(sum) * sum;
	}
	long key = long(ind) << 32 | sum;
	long ans = memo[key];
	if (ans) {
		return decode(ans);
	}
	long cdp = min(dp(ind + 1, sum + psum[ind]) + n * sq(psum[ind]),
				   dp(ind + 1, sum + (ind + 1) * arr[ind]) +
					   2 * n * sum * arr[ind] + n * (ind + 1) * sq(arr[ind]));
	memo[key] = encode(cdp);
	return cdp;
}

void solve() {
	cin >> n;
	int in[n];
	for (auto& a : in) {
		cin >> a;
	}
	for (int i = 0; i < n - 1; i++) {
		arr[i] = in[i + 1] - in[i];
	}
	sort(arr, arr + n - 1);
	partial_sum(arr, arr + n - 1, psum);
	cout << dp(0, 0) << endl;
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