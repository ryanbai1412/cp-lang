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

const int mod = 1e9 + 7;
const long pcomp1[] = {0, 0, 12, 112, 912, 7136, 56768, 453504, 3626752},
		   pcomp2[] = {0, 0, 36, 336, 2688, 21312, 170112, 1360128, 10879488};

long bpow(long base, long exp) {
	long ans = 1;
	while (exp) {
		if (exp & 1) {
			ans = (ans * base) % mod;
		}
		base = (base * base) % mod;
		exp >>= 1;
	}
	return ans;
}

void solve() {
	int n, m;
	cin >> n >> m;
	if (n > m) {
		swap(n, m);
	}
	if (n == 1) {
		cout << bpow(2, m) << endl;
	} else if (n == m) {
		cout << pcomp1[n] << endl;
	} else {
		cout << (pcomp2[n] * bpow(3, m - n - 1)) % mod << endl;
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