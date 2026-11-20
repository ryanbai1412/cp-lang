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

template <long mod>
constexpr long bpow(long base, long exp) {
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

template <long mod>
constexpr long inv(long x) {
	return bpow<mod>(x, mod - 2);
}

template <long mod, int maxn>
struct PPow {
	long pcomp[maxn + 1] {};

	explicit constexpr PPow(long base) {
		pcomp[0] = 1;
		for (int i = 0; i < maxn; i++) {
			pcomp[i + 1] = (pcomp[i] * base) % mod;
		}
	}

	long operator[](int ind) const {
		return pcomp[ind];
	}
};

const int maxn = 400;

template <long mod>
struct Hasher {
	static constexpr long base = 3, ibase = inv<mod>(base), rmul = bpow<mod>(base, maxn);
	static constexpr PPow<mod, maxn * maxn> basep {base}, ibasep {ibase};

	static int ctoi(char c) {
		return (c == '1') + 1;
	}

	long psum[maxn][maxn] {};

	explicit Hasher(const vector<string>& arr) {
		int n = sz(arr), m = sz(arr[0]);
		long crmul = 1;
		for (int i = 0; i < n; i++) {
			long cmul = crmul;
			for (int j = 0; j < m; j++) {
				psum[i + 1][j + 1] =
					(psum[i][j + 1] + psum[i + 1][j] - psum[i][j] + ctoi(arr[i][j]) * cmul + mod) %
					mod;
				cmul = (cmul * base) % mod;
			}
			crmul = (crmul * rmul) % mod;
		}
	}

	long query(int x1, int y1, int x2, int y2) const {
		long raw = (psum[x2][y2] - psum[x2][y1] - psum[x1][y2] + psum[x1][y1] + mod + mod) % mod;
		raw = (raw * ibasep[x1 * maxn]) % mod;
		raw = (raw * ibasep[y1]) % mod;
		return raw;
	}
};

const long mod = 1e9 + 7;

void solve() {
	int n, m;
	cin >> n >> m;
	vector<string> arr(n);
	for (auto& a : arr) {
		cin >> a;
	}
	vector<string> iarr(arr);
	for (int i = 0; i < n; i++) {
		for (int j = 0; j < m; j++) {
			iarr[i][j] = arr[n - i - 1][m - j - 1];
		}
	}
	for (auto& a : iarr) {
		cerr << a << endl;
	}
	Hasher<mod> hasher(arr), ihasher(iarr);
	for (int x = min(n, m); x >= 2; x--) {
		for (int i = 0; i + x <= n; i++) {
			for (int j = 0; j + x <= m; j++) {
				int ni = i + x, nj = j + x;
				ni = n - ni;
				nj = m - nj;
				if (hasher.query(i, j, i + x, j + x) == ihasher.query(ni, nj, ni + x, nj + x)) {
					dbg(i, j, ni, nj);
					cout << x << endl;
					return;
				}
			}
		}
	}
	cout << -1 << endl;
}

int main() {
	cin.tie(nullptr);
	ios_base::sync_with_stdio(false);
	cin.exceptions(ios::failbit);
	int t = 1;
	// cin >> t;
	for (int _ = 1; _ <= t; _++) {
		dbg(_);
		// cout << "Case #" << _ << ": ";
		solve();
	}
}