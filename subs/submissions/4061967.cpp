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

const long mod = 998244353;
const int maxn = 30, maxm = 100;

long fact[maxm + 1], ifact[maxm + 1];
long dp[maxn + 1][maxm + 2][16][maxn + 1] {};

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

void pcomp() {
	fact[0] = ifact[0] = 1;
	for (int i = 1; i <= maxm; i++) {
		fact[i] = (fact[i - 1] * i) % mod;
		ifact[i] = bpow(fact[i], mod - 2);
	}
}

namespace s1 {
void solve(int n, int m, int k) {
	int arr[m + 1];
	for (auto& a : arr) {
		cin >> a;
	}
	for (auto& a : dp[n]) {
		for (int bits = 0; bits < 16; bits++) {
			int cur = __builtin_popcount(bits);
			for (int i = 0; i + cur <= k; i++) {
				a[bits][i] = 1;
			}
		}
	}
	for (int i = n - 1; i >= 0; i--) {
		for (int cur_min = 0; cur_min <= m; cur_min++) {
			for (int prev_bit = 0; prev_bit < 16; prev_bit++) {
				for (int k = 0; k <= 1; k++) {
					long& ans = dp[i][cur_min][prev_bit][k];
					int prev_bit_bitcount = __builtin_popcount(prev_bit);
					for (int new_val = cur_min; new_val <= m; new_val++) {
						int shift = new_val - cur_min;
						int new_bits = shift > 5 ? 0 : prev_bit >> shift;
						int new_k = k + prev_bit_bitcount -
									__builtin_popcount(new_bits);
						long cur_mul = 1;
						for (int new_index = i + 1; new_index <= n;
							 new_index++) {
							cur_mul = (cur_mul * arr[new_val]) % mod;
							int new_count = new_index - i;
							int cur_bits = new_bits + new_count;
							long next_val =
								cur_mul *
								dp[new_index][new_val + 1][cur_bits >> 1]
								  [new_k + (cur_bits & 1)];
							next_val %= mod;
							ans = (ans + next_val * ifact[new_count]) % mod;
						}
					}
				}
			}
		}
	}
	cout << (fact[n] * dp[0][0][0][0]) % mod << endl;
}
}  // namespace s1

void solve() {
	int n, m, k;
	cin >> n >> m >> k;
	if (k == 1) {
		s1::solve(n, m, k);
		return;
	}
	int arr[m + 1];
	for (auto& a : arr) {
		cin >> a;
	}
	for (auto& a : dp[n]) {
		for (int bits = 0; bits < 16; bits++) {
			int cur = __builtin_popcount(bits);
			for (int i = 0; i + cur <= k; i++) {
				a[bits][i] = 1;
			}
		}
	}
	for (int i = n - 1; i >= 0; i--) {
		for (int cur_min = 0; cur_min <= m; cur_min++) {
			for (int prev_bit = 0; prev_bit < 16; prev_bit++) {
				for (int k = 0; k <= n; k++) {
					long& ans = dp[i][cur_min][prev_bit][k];
					int prev_bit_bitcount = __builtin_popcount(prev_bit);
					for (int new_val = cur_min; new_val <= m; new_val++) {
						int shift = new_val - cur_min;
						int new_bits = shift > 5 ? 0 : prev_bit >> shift;
						int new_k = k + prev_bit_bitcount -
									__builtin_popcount(new_bits);
						long cur_mul = 1;
						for (int new_index = i + 1; new_index <= n;
							 new_index++) {
							cur_mul = (cur_mul * arr[new_val]) % mod;
							int new_count = new_index - i;
							int cur_bits = new_bits + new_count;
							long next_val =
								cur_mul *
								dp[new_index][new_val + 1][cur_bits >> 1]
								  [new_k + (cur_bits & 1)];
							next_val %= mod;
							ans = (ans + next_val * ifact[new_count]) % mod;
						}
					}
				}
			}
		}
	}
	cout << (fact[n] * dp[0][0][0][0]) % mod << endl;
}

int main() {
	cin.tie(nullptr);
	ios_base::sync_with_stdio(false);
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