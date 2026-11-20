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
	uint64_t operator()(uint64_t x) const {
		x ^= x << 49;
		x ^= x >> 23;
		x ^= x << 37;
		return x;
	}
};

const int csize = 3e7 + 5;

long cache[csize];
__gnu_pbds::gp_hash_table<int, long> memo;

long dp(int x) {
	if (x < csize) {
		if (cache[x] != -1) {
			return cache[x];
		}
	} else {
		auto it = memo.find(x);
		if (it != memo.end()) {
			return it->second;
		}
	}
	int bord = floor(sqrt(x));
	long ans = 0;
	for (int i = 1; i <= bord; i++) {
		int r = x / i;
		int l = x / (i + 1) + 1;
		ans += (r - l + 1) * dp(i);
	}
	for (int i = 2; i <= bord + 1; i++) {
		int j = x / i;
		if (bord < j && j <= x) {
			dbg(x, j);
			ans += dp(j);
		}
	}
	if (x < csize) {
		cache[x] = ans;
	} else {
		memo[x] = ans;
	}
	return ans;
}

void solve() {
	int n;
	cin >> n;
	memset(cache, -1, sizeof(cache));
	cache[1] = 1;
	cout << dp(n) << endl;
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