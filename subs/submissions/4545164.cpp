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
	auto csmall = [&](int x) -> int {
		return int(lower_bound(begin(small), end(small), x) - small.begin());
	};
	auto cbig = [&](int x) -> int {
		return int(lower_bound(begin(big), end(big), x) - big.begin());
	};
	int n;
	cin >> n;
	int arr[n], bv[maxa + 1], fact[maxa + 1] {}, cnt[maxa + 1] {};
	for (auto& a : arr) {
		cin >> a;
	}
	for (auto& a : arr) {
		cnt[a]++;
	}
	for (int i = 1; i <= maxa; i++) {
		auto cfact = factor(i);
		if (sz(cfact) && cfact.back() >= psep) {
			bv[i] = cbig(cfact.back());
			cfact.pop_back();
		} else {
			bv[i] = -1;
		}
		for (auto& a : cfact) {
			fact[i] |= 1 << csmall(a);
		}
	}
	long ca[1 << maxs], cb[1 << maxs][maxb];
	int cnta[1 << maxs] {};
	vector<int> opta[1 << maxs];
	for (int i = 0; i < (1 << maxs); i++) {
		for (int j = 1; j <= maxa; j++) {
			if ((fact[j] & i) == 0) {
				opta[i].push_back(j);
				cnta[i] += cnt[j];
			}
		}
	}
	for (int i = 0; i < (1 << maxs); i++) {
		ca[i] = pow2[n] - pow2[cnta[i]];
	}
	for (int i = 0; i < (1 << maxs); i++) {
		int cntb[maxb] {};
		for (auto& k : opta[i]) {
			if (bv[k] >= 0) {
				cntb[bv[k]]++;
			}
		}
		for (int j = 0; j < maxb; j++) {
			int cur = cntb[j];
			cb[i][j] = (pow2[cur] - 1) * pow2[cnta[i] - cur] % mod;
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
		bool picked[maxb] {};
		for (auto& a : carr) {
			if (a < psep) {
				sfact |= 1 << csmall(a);
			} else {
				picked[cbig(a)] = true;
			}
		}
		long ans = 0;
		auto upd = [&](int mask, long x) -> void {
			if (__builtin_popcount(mask) & 1) {
				ans += x;
			} else {
				ans -= x;
			}
		};
		for (int i = 0; i < (1 << maxs); i++) {
			if ((i & sfact) == i) {
				long dpa = ca[i], dpb = 0, wa = 1, wb = 0;
				for (int j = 0; j < maxb; j++) {
					if (picked[j]) {
						long ndpa = dpa + dpb + cb[i][j] * wb, ndpb = dpb + dpa + cb[i][j] * wa;
						long nwa = wa + wb, nwb = wb + wa;
						ndpa %= mod;
						ndpb %= mod;
						nwa %= mod;
						nwb %= mod;
						tie(dpa, dpb, wa, wb) = tie(ndpa, ndpb, nwa, nwb);
					}
				}
				dbg(i, dpa, dpb);
				upd(i, dpa);
				upd(i ^ 1, dpb);
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