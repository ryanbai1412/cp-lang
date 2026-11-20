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

const long mod = 1e9 + 9;

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

long inv(long x) {
	return bpow(x, mod - 2);
}

const long base = 257, ibase = inv(base);

vector<long> ipow;

void pcomp() {
	long cur = 1;
	ipow.push_back(1);
	for (int i = 1; i < int(1e5); i++) {
		ipow.push_back(cur = (cur * ibase) % mod);
	}
}

struct H {
	vector<long> psum;

	H(const vector<long>& arr) {
		psum.push_back(0);
		long x = 1;
		for (auto& a : arr) {
			psum.push_back((psum.back() + a * x) % mod);
			x = (x * base) % mod;
		}
	}

	long query(int l, int r) const {
		return (psum[r + 1] - psum[l] + mod) * ipow[l] % mod;
	}
};

void solve() {
	int n, m, k;
	cin >> n >> m >> k;
	char arr[n][m];
	for (auto& a : arr) {
		for (auto& b : a) {
			cin >> b;
		}
	}
	vector<H> hori;
	for (int i = 0; i < n; i++) {
		vector<long> cur(m);
		for (int j = 0; j < m; j++) {
			cur[j] = arr[i][j] - 'a' + 1;
		}
		hori.emplace_back(cur);
	}
	set<long> ans;
	for (int i = 0; i + k <= m; i++) {
		vector<long> cur(n);
		for (int j = 0; j < n; j++) {
			cur[j] = hori[j].query(i, i + k - 1);
		}
		H h(cur);
		for (int j = 0; j + k <= n; j++) {
			ans.insert(h.query(j, j + k - 1));
		}
	}
	cout << sz(ans) << endl;
}

int main() {
	cin.tie(nullptr);
	ios_base::sync_with_stdio(false);
	cin.exceptions(ios::failbit);
	pcomp();
	solve();
}