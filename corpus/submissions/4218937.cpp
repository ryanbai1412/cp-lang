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
	if (n == 1) {
		cout << 1 << endl;
	} else if (n == 2 || n == 3) {
		cout << -1 << endl;
	} else if (n == 4) {
		cout << "3 1 4 2" << endl;
	} else {
		int limit = 40;
		if (n % 10 == 0) {
			limit = 25;
		}
		int vis[n] {};
		int ct = 0;
		int ind = int(upper_bound(begin(primes), end(primes), n) - primes.begin());
		for (int ai = 0; ai < ind; ai++) {
			int a = primes[ai];
			for (int bi = ind - 1; bi >= 0 && bi >= ind - limit; bi--) {
				int b = primes[bi];
				ct++;
				auto check = [&](int ind) -> bool {
					return 0 <= ind && ind < n && vis[ind] != ct;
				};
				{
					int cur = -1;
					for (int i = 0; i < n; i++) {
						if (!i) {
							cur = 0;
						} else {
							if (check(cur + a)) {
								cur += a;
							} else if (check(cur - b)) {
								cur -= b;
							} else {
								goto loop;
							}
						}
						vis[cur] = ct;
					}
				}
				ct++;
				{
					int cur = -1;
					for (int i = 0; i < n; i++) {
						if (!i) {
							cur = 0;
						} else {
							if (check(cur + a)) {
								cur += a;
							} else if (check(cur - b)) {
								cur -= b;
							} else {
								goto loop;
							}
						}
						cout << cur + 1 << " \n"[i == n - 1];
						vis[cur] = ct;
					}
				}
				return;
			loop:;
			}
		}
		assert(false);
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