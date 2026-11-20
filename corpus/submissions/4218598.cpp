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

const int maxn = 2e6;

bool prime[maxn + 1];
vector<int> primes;

void pcomp() {
	memset(prime, 1, sizeof(prime));
	prime[0] = prime[1] = false;
	for (int i = 2; i <= maxn; i++) {
		if (prime[i]) {
			primes.push_back(i);
			if (long(i) * i <= maxn) {
				for (int j = i * i; j <= maxn; j += i) {
					prime[j] = false;
				}
			}
		}
	}
}

void solve() {
	int n;
	cin >> n;
	int dp[1 << n][n] {};
	for (int i = (1 << n) - 1; i >= 0; i--) {
		for (int j = 0; j < n; j++) {
			if ((i >> j) & 1) {
				for (int k = 0; k < n; k++) {
					if (!((i >> k) & 1)) {
						int add = 0;
						if (prime[abs(j - k)]) {
							add = abs(j - k);
						}
						dp[i][j] = max(dp[i][j], add + dp[i | (1 << k)][k]);
					}
				}
			}
		}
	}
	pair<int, int> opt {0, 0};
	for (int i = 0; i < n; i++) {
		opt = max(opt, pair<int, int> {dp[1 << i][i], i});
	}
	dbg(opt.first);
	int j = opt.second;
	int i = 1 << j;
	cout << j + 1;
	for (int _ = 0; _ < n - 1; _++) {
		for (int k = 0; k < n; k++) {
			if (!((i >> k) & 1)) {
				int add = 0;
				if (prime[abs(j - k)]) {
					add = abs(j - k);
				}
				if (add + dp[i | (1 << k)][k] == dp[i][j]) {
					i ^= 1 << k;
					j = k;
					break;
				}
			}
		}
		cout << " " << j + 1;
	}
	cout << endl;
}

int main() {
	cin.tie(nullptr);
	ios_base::sync_with_stdio(false);
	cin.exceptions(ios::failbit);
	pcomp();
	int t = 1;
	// cin >> t;
	for (int _ = 1; _ <= t; _++) {
		dbg(_);
		// cout << "Case #" << _ << ": ";
		solve();
	}
}