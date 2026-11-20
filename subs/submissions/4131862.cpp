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

const int maxn = 50;

using mat = array<array<long, maxn>, maxn>;

const long mod = 1e9 + 7;

mat operator*(const mat &a, const mat &b) {
	mat ret {};
	for (int i = 0; i < maxn; i++) {
		for (int j = 0; j < maxn; j++) {
			for (int k = 0; k < maxn; k++) {
				ret[j][k] = (ret[j][k] + a[j][i] * b[i][k]) % mod;
			}
		}
	}
	return ret;
}

mat bpow(mat m, long exp) {
	mat ans {};
	for (int i = 0; i < maxn; i++) {
		ans[i][i] = 1;
	}
	while (exp) {
		if (exp & 1) {
			ans = ans * m;
		}
		m = m * m;
		exp >>= 1;
	}
	return ans;
}

void solve() {
	int n;
	long k;
	cin >> n >> k;
	mat m {};
	for (int i = 0; i < n; i++) {
		for (int j = 0; j < n; j++) {
			cin >> m[i][j];
		}
	}
	mat ans = bpow(m, k);
	long cans = 0;
	for (auto& a : ans) {
		for (auto& b : a) {
			cans += b;
		}
	}
	cout << cans % mod << endl;
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