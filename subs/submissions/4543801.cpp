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

const int maxp = 1e6 + 5, maxa = 2e3, maxs = 13, maxb = 303 - maxs;
const long mod = 998244353;

vector<int> primes;
long pow2[maxp];

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
	pow2[0] = 1;
	for (int i = 1; i < maxp; i++) {
		pow2[i] = (pow2[i - 1] * 2) % mod;
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

void solve() {
	assert(maxs + maxb == sz(primes));
	vector<int> small(primes.begin(), primes.begin() + maxs),
		big(primes.begin() + maxs, primes.end());
	int psep = big[0];
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
		if (cfact.back() >= psep) {
			bv[i] = cbig(cfact.back());
			cfact.pop_back();
		} else {
			bv[i] = 0;
		}
		for (auto& a : cfact) {
			fact[i] |= 1 << csmall(a);
		}
	}
	long comp[1 << maxs][maxb + 1];
	for (int i = 0; i < (1 << maxs); i++) {
		for (int j = 0; j <= maxb; j++) {
			int cnt = 0;
			for (int k = 0; k < n; k++) {
				cnt += ((fact[k] & i) == 0) && (!j || bv[k] != j);
			}
			comp[i][j] = pow2[cnt];
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
			if (a < psep) {
				sfact |= 1 << csmall(a);
			} else {
				bigs.push_back(cbig(a));
			}
		}
		sort(begin(bigs), end(bigs));
		bigs.erase(unique(begin(bigs), end(bigs)), bigs.end());
		long ans = 0;
		auto upd = [&](int mask, long x) -> void {
			if (__builtin_popcount(mask) & 1) {
				ans -= x;
			} else {
				ans += x;
			}
		};
		for (int i = 0; i < (1 << maxs); i++) {
			if ((i & sfact) == i) {
				upd(i, comp[i][0]);
				for (auto& a : bigs) {
					dbg(comp[i][a]);
					upd(i ^ 1, comp[i][a]);
				}
			}
		}
		ans %= mod;
		if (ans < 0) {
			ans += mod;
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