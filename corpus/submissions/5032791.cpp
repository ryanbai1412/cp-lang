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

const int maxn = 1e5;

bool prime[maxn + 1], dp[maxn + 1];
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
	for (int i = 1; i <= maxn; i++) {
		for (auto& a : primes) {
			if (a <= i) {
				dp[i] |= !dp[i - a];
			} else {
				break;
			}
		}
		if (!dp[i]) {
			dbg(i + 1);
		}
	}
}

void solve() {
	int n;
	cin >> n;
	if (n == 1 || n == 2) {
		cout << -1 << endl;
	} else if (!dp[n - 1]) {
		int arr[n];
		auto check = [&]() -> bool {
			int iarr[n];
			for (int i = 0; i < n; i++) {
				iarr[arr[i] - 1] = i;
			}
			bool dp[n] {};
			for (int i = 0; i < n; i++) {
				auto upd = [&](int x) -> void {
					if (0 <= x && x < n && arr[x] - 1 < i) {
						dp[i] |= !dp[arr[x] - 1];
					}
				};
				for (auto& a : primes) {
					if (a < n) {
						upd(iarr[i] - a);
						upd(iarr[i] + a);
					} else {
						break;
					}
				}
			}
			return dp[n - 1];
		};
		auto print = [&]() -> void {
			for (int i = 0; i < n; i++) {
				cout << arr[i] << " \n"[i == n - 1];
			}
		};
//		iota(arr, arr + n, 1);
//		for (int i = 0; i < 10; i++) {
//			if (check()) {
//				print();
//				return;
//			}
//			next_permutation(arr, arr + n);
//		}
//		assert(false);
		if (n == 11 || n == 36 || n == 311) {
			int arr[n];
			iota(arr, arr + n, 1);
			for (int i = 0; i < 4; i++) {
				next_permutation(arr, arr + n);
			}
			for (int i = 0; i < n; i++) {
				cout << arr[i] << " \n"[i == n - 1];
			}
		} else {
			for (int i = 1; i <= n - 2; i++) {
				cout << i << " ";
			}
			cout << n << " " << n - 1 << endl;
		}
	} else {
		for (int i = 1; i <= n; i++) {
			cout << i << " \n"[i == n];
		}
	}
}

int main() {
	cin.tie(nullptr);
	ios_base::sync_with_stdio(false);
	cin.exceptions(ios::failbit);
	pcomp();
	int t = 1;
	cin >> t;
	for (int _ = 1; _ <= t; _++) {
		dbg(_);
		// cout << "Case #" << _ << ": ";
		solve();
	}
}