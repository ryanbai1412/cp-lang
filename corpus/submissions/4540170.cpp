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
#define dbg(...)
#endif

#define endl "\n"
#define long int64_t
#define sz(x) int((x).size())

const int maxa = 2e3, maxs = 14, maxb = 303 - maxs;
const long mod = 998244353;

vector<int> primes;

void pcomp() {
	bool prime[maxa + 1];
	memset(prime, 1, sizeof(prime));
	for (int i = 2; i <= maxa; i++) {
		if (prime[i]) {
			primes.push_back(i);
			for (int j = i * i; j <= maxa; j += i) {
				prime[j] = false;
			}
		}
	}
}

vector<int> factor(int x) {
	vector<int> ans;
	for (auto& a : primes) {
		if (x % a == 0) {
			ans.push_back(a);
			while (x % a == 0) {
				x /= a;
			}
		}
	}
	return ans;
}

namespace s1 {

void solve(int n) {
	int arr[n];
	for (auto& a : arr) {
		cin >> a;
	}
	int q;
	cin >> q;
	while (q--) {
		int m;
		cin >> m;
		int carr[m];
		for (auto& a : carr) {
			cin >> a;
		}
		int ans = 0;
		for (int i = 0; i < (1 << n); i++) {
			for (auto& a : carr) {
				bool ok = false;
				for (int j = 0; j < n; j++) {
					if ((i >> j) & 1) {
						ok |= arr[j] % a == 0;
					}
				}
				if (!ok) {
					goto loop;
				}
			}
			ans++;
		loop:;
		}
		cout << ans << endl;
	}
}

}  // namespace s1

void solve() {
	int n;
	cin >> n;
	if (n <= 10) {
		s1::solve(n);
		return;
	}
	auto csmall = [&](int x) -> int {
		return int(lower_bound(begin(primes), end(primes), x) - primes.begin());
	};
	long p2[3001];
	p2[0] = 1;
	for (int i = 0; i < 3000; i++) {
		p2[i + 1] = (p2[i] * 2) % mod;
	}
	for (auto& a : p2) {
		a--;
	}
	int arr[n];
	for (auto& a : arr) {
		cin >> a;
	}
	int cnt[2001] {};
	for (auto& a : arr) {
		cnt[a]++;
	}
	int fact[n] {};
	for (int i = 0; i < n; i++) {
		auto cx = factor(arr[i]);
		for (auto& a : cx) {
			fact[i] |= 1 << csmall(a);
		}
	}
	long dp[1 << maxs] {};
	dp[0] = 1;
	for (int i = 1; i <= 2000; i++) {
		for (int j = (1 << maxs) - 1; j >= 0; j--) {
			dp[j | i] = (dp[j | i] + dp[j] * p2[cnt[i]]) % mod;
		}
	}
	for (auto& a : fact) {
		for (int j = (1 << maxs) - 1; j >= 0; j--) {
			auto& nxt = dp[j | a];
			nxt += dp[j];
			if (nxt >= mod) {
				nxt -= mod;
			}
		}
	}
	int q;
	cin >> q;
	while (q--) {
		int m;
		cin >> m;
		int carr[m];
		for (auto& a : carr) {
			cin >> a;
		}
		bool bad = false;
		int sfact = 0;
		vector<int> bigs;
		for (auto& a : carr) {
			if (a < 45) {
				sfact |= 1 << csmall(a);
			} else {
				bad = true;
			}
		}
		if (bad) {
			cout << 0 << endl;
			continue;
		}
		cout << dp[sfact] << endl;
	}
}

int main() {
	cin.tie(nullptr);
	ios_base::sync_with_stdio(false);
	cin.exceptions(ios::failbit);
	pcomp();
	solve();
}