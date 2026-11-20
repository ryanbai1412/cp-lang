#include "bits/extc++.h"

using namespace std;

template<class T>
using mpq = priority_queue<T, vector<T>, greater<>>;

template<class T, class U = less<T>>
using rt = __gnu_pbds::tree<T, __gnu_pbds::null_type, U, __gnu_pbds::rb_tree_tag, __gnu_pbds::tree_order_statistics_node_update>;

template<class T>
void dbgh(const T &t) {
	cerr << t << endl;
}

template<class T, class ...U>
void dbgh(const T &t, const U &...u) {
	cerr << t << " | ";
	dbgh(u...);
}

#ifdef DEBUG
#define dbg(...) cerr << "L" << __LINE__ << " [" << #__VA_ARGS__ << "]" << ": "; dbgh(__VA_ARGS__)
#else
#define cerr if(false) cerr
#define dbg(...) 1412
#endif

//imagine a language where int = long
#define long int64_t

//typing too hard
#define endl "\n"

#define sz(x) static_cast<int>((x).size())

#define inline inline __attribute__((always_inline))

template<int mod>
constexpr long bpow(long base, long exp) {
	long ans = 1;
	while(exp) {
		if(exp & 1) {
			ans = (ans * base) % mod;
		}
		base = (base * base) % mod;
		exp >>= 1;
	}
	return ans;
}

template<int mod>
constexpr long inv(long x) {
	return bpow<mod>(x, mod - 2);
}

template<int mod>
struct Hash {
	static constexpr int base = 29, ibase = inv<mod>(base);
	vector<long> pow, ipow, psum;

	Hash(const string &s) {
		int n = sz(s);
		pow.resize(n + 1);
		ipow.resize(n + 1);
		psum.resize(n + 1);
		pow[0] = ipow[0] = 1;
		for(int i = 0; i < n; i++) {
			pow[i + 1] = (pow[i] * base) % mod;
			ipow[i + 1] = (ipow[i] * ibase) % mod;
			psum[i + 1] = (psum[i] + (s[i] - 'a') * pow[i]) % mod;
		}
	}

	long hash(int l, int r) {
		return ((psum[r] - psum[l] + mod) * ipow[l]) % mod;
	}
};

void solve() {
	int n;
	cin >> n;
	string s;
	cin >> s;
	long arr[n];
	for(int i = 0; i < n; i++) {
		cin >> arr[i];
	}
	Hash<int(1e9 + 7)> hasher {s};
	auto eq = [&](int i, int j, int r) -> bool {
		return i + r <= n && j + r <= n && hasher.hash(i, i + r) == hasher.hash(j, j + r);
	};
	auto meq = [&](int i, int j) -> int {
		int l = 0, r = n - 1;
		while(l < r) {
			int mid = (l + r + 1) / 2;
			if(eq(i, j, mid)) {
				l = mid;
			}else {
				r = mid - 1;
			}
		}
		return l;
	};
	long pways[n + 1] {}, pmax[n + 1];
	memset(pmax, 0x80, sizeof(pmax));
	for(int i = 0; i < n; i++) {
		for(int j = i + 1; j < n; j++) {
			int c = meq(i, j);
			pways[c]++;
			pmax[c] = max(pmax[c], arr[i] * arr[j]);
		}
	}
	for(int i = n - 1; i >= 0; i--) {
		pways[i] += pways[i + 1];
		pmax[i] = max(pmax[i], pmax[i + 1]);
	}
	for(int i = 0; i < n; i++) {
		if(pways[i]) {
			cout << pways[i] << " " << pmax[i] << endl;
		}else {
			cout << 0 << " " << 0 << endl;
		}
	}
}

int main() {
	cin.tie(0)->sync_with_stdio(0);
	cin.exceptions(ios::failbit);
#ifdef LOCAL
	freopen("input.txt", "r", stdin);
#endif
	int t = 1;
//	cin >> t;
	for(int _ = 1; _ <= t; _++) {
		dbg(_);
//		cout << "Case #" << _ << ": ";
		solve();
	}
}