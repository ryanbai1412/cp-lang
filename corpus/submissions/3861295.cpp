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

typedef uint64_t ull;
const int N = 400000;
bool arr[N + 1][256];

ull myRand(ull &k1, ull &k2) {
	ull k3 = k1, k4 = k2;
	k1 = k4;
	k3 ^= (k3 << 23);
	k2 = k3 ^ k4 ^ (k3 >> 17) ^ (k4 >> 26);
	return k2 + k4;
}

void gen(int n, ull a1, ull a2) {
	for(int i = 1; i <= n; i++)
		for(int j = 0; j < 256; j++)
			arr[i][j] = (myRand(a1, a2) & (1ull << 32)) ? 1 : 0;
}

ulong nibbleToInt(char c) {
	if(isdigit(c)) {
		return c - '0';
	}
	return 10 + c - 'A';
}

void solve() {
	int n, q;
	ull a1, a2;
	cin >> n >> q >> a1 >> a2;
	dbg(a1, a2);
	gen(n, a1, a2);
	for(int i = 1; i<= n; i++) {
		for(int j = 0; j < 256; j++) {
			cerr << arr[i][j];
		}
		cerr << endl;
	}
	dbg("");
	bool lans = false;
	while(q--) {
		string s;
		int k;
		cin >> s >> k;
		bool bits[256] {};
		for(int i = 0; i < 64; i++) {
			ulong cur = nibbleToInt(s[i]);
			for(int j = 3; j >= 0; j--) {
				bits[i * 4 + 3 - j] = (cur >> j) & 1;
			}
		}
		for(auto &a: bits) {
			a ^= lans;
			cerr << a;
		}
		cerr << endl;
		bits[255] ^= lans;
		pair<int, int> mk {1e9, 0};
		for(int i = 1; i <= n; i++) {
			int cur = 0;
			for(int j = 0; j < 256; j++) {
				cur += arr[i][j] ^ bits[j];
			}
			mk = min(mk, pair<int, int> {cur, i});
		}
		dbg(mk.first, mk.second);
		lans = mk.first <= k;
		cout << lans << endl;
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