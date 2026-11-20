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

long dpa[maxb + 1][1 << maxs], dpb[maxb + 1][1 << maxs];

void solve() {
	dbg(maxs, maxb, sz(primes), primes[maxs]);
	assert(maxs + maxb == sz(primes));
	vector<int> small(primes.begin(), primes.begin() + maxs),
		big(primes.begin() + maxs, primes.end());
	big.insert(big.begin(), 1);
	auto csmall = [&](int x) -> int {
		return int(lower_bound(begin(small), end(small), x) - small.begin());
	};
	auto cbig = [&](int x) -> int {
		return int(lower_bound(begin(big), end(big), x) - big.begin());
	};
	int n;
	cin >> n;
	int arr[n], bv[n], fact[n] {};
	for (auto& a : arr) {
		cin >> a;
	}
	for (int i = 0; i < n; i++) {
		auto cfact = factor(arr[i]);
		if (sz(cfact) && cfact.back() > 45) {
			bv[i] = cbig(cfact.back());
			cfact.pop_back();
		} else {
			bv[i] = 0;
		}
		for (auto& a : cfact) {
			fact[i] |= 1 << csmall(a);
		}
		dbg(i, arr[i], bv[i], fact[i]);
	}
	int bcnt[maxb + 1] {};
	for (auto& a : bv) {
		bcnt[a]++;
	}
	long bways[maxb + 1];
	for (int i = 0; i <= maxb; i++) {
		long cur = 1;
		for (int j = 0; j < bcnt[i]; j++) {
			cur = (cur * 2) % mod;
		}
		bways[i] = (cur - 1 + mod) % mod;
	}
	for (int i = 0; i < maxb; i++) {
		{
			dpa[i][0] = 1;
			for (auto& a : fact) {
				for (int j = (1 << maxs) - 1; j >= 0; j--) {
					auto& nxt = dpa[i][j | a];
					nxt += dpa[i][j];
					if (nxt >= mod) {
						nxt -= mod;
					}
				}
			}
		}
		{
			vector<int> cur;
			for (int j = 0; j < n; j++) {
				if (bv[j] != i) {
					cur.push_back(fact[j]);
				}
			}
			dpb[i][0] = 1;
			for (auto& a : fact) {
				for (int j = (1 << maxs) - 1; j >= 0; j--) {
					auto& nxt = dpb[i][j | a];
					nxt += dpb[i][j];
					if (nxt >= mod) {
						nxt -= mod;
					}
				}
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
		int sfact = 0;
		vector<int> bigs;
		for (auto& a : carr) {
			if (a < 45) {
				sfact |= 1 << csmall(a);
			} else {
				bigs.push_back(cbig(a));
			}
		}
		sort(begin(bigs), end(bigs));
		bigs.erase(unique(begin(bigs), end(bigs)), bigs.end());
		long ans = 0;
		for (int i = 0; i < (1 << maxs); i++) {
			if ((i & sfact) == sfact) {
				ans += dpa[0][i];
			}
		}
		dbg(ans);
		ans %= mod;
		for (auto& a : bigs) {
			long ca = 0, cb = 0;
			for (int i = 0; i < (1 << maxs); i++) {
				if ((i & sfact) == sfact) {
					ca += dpa[0][i];
					cb += dpb[0][i];
				}
			}
			ca %= mod;
			cb %= mod;
			long cx = (ca - cb + mod) + cb * bways[a];
			cx %= mod;
			ans = (ans * cx) % mod;
		}
		cout << ans << endl;
	}
}

int main() {
	cin.tie(nullptr);
	ios_base::sync_with_stdio(false);
	cin.exceptions(ios::failbit);
	pcomp();
	solve();
}