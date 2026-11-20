#pragma GCC optimize("Ofast,unroll-loops,no-stack-protector")
#pragma GCC target("avx,avx2")

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
#define ulong uint64_t

//typing too hard
#define endl "\n"

#define sz(x) static_cast<int>((x).size())

#define inline inline __attribute__((always_inline))

const int maxn = 400000;

ulong dict[maxn][4];

using ull = ulong;
ull myRand(ull &k1, ull &k2) {
	ull k3 = k1, k4 = k2;
	k1 = k4;
	k3 ^= (k3 << 23);
	k2 = k3 ^ k4 ^ (k3 >> 17) ^ (k4 >> 26);
	return k2 + k4;
}

void gen(int n, ull a1, ull a2) {
	for(int i = 0; i < n; i++) {
		for(int j = 0; j < 4; j++) {
			for(int k = 0; k < 64; k++) {
				dict[i][j] = (dict[i][j] << 1) | ((myRand(a1, a2) >> 32) & 1);
			}
		}
	}
}

ulong nibbleToInt(char c) {
	if(isdigit(c)) {
		return c - '0';
	}
	return 10 + c - 'A';
}

void solve() {
	int n, q;
	ulong a1, a2;
	cin >> n >> q >> a1 >> a2;
	gen(n, a1, a2);
	bool ans = false;
	while(q--) {
		string s;
		int k;
		cin >> s >> k;
		ulong arr[4] {};
		for(int i = 0; i < 4; i++) {
			for(int j = 0; j < 16; j++) {
				arr[i] = (arr[i] << 4) | nibbleToInt(s[i * 16 + j]);
			}
			if(ans) {
				arr[i] = ~arr[i];
			}
		}
		ans = false;
		for(int i = 0; i < n; i++) {
			ans |= (__builtin_popcountll(arr[0] ^ dict[i][0])
					+ __builtin_popcountll(arr[1] ^ dict[i][1])
					+ __builtin_popcountll(arr[2] ^ dict[i][2])
					+ __builtin_popcountll(arr[3] ^ dict[i][3])) <= k;
		}
		cout << ans << endl;
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