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

const int maxn = 1e6, logn = 20;
const long mod = 1e9 + 7;

int lift[logn][maxn];

void solve() {
	string s;
	cin >> s;
	int n = sz(s);
	int next[n] {}, cnt[n] {};
	for(int i = 1; i < n; i++) {
		int j = next[i - 1];
		while(j && s[i] != s[j]) {
			j = next[j];
		}
		if(s[i] == s[j]) {
			next[i] = j + 1;
			cnt[i] = cnt[j] + 1;
		}
		dbg(i, next[i], cnt[i]);
	}
	for(auto &a: next) {
		a--;
	}
	memcpy(lift[0], next, sizeof(next));
	for(int i = 0; i < logn - 1; i++) {
		for(int j = 0; j < n; j++) {
			if(lift[i][j] == -1) {
				lift[i + 1][j] = -1;
			}else {
				lift[i + 1][j] = lift[i][lift[i][j]];
			}
		}
	}
	long ans = 1;
	for(int i = 1; i < n; i++) {
		int u = i;
		for(int j = logn - 1; j >= 0; j--) {
			int nu = lift[j][u];
			if(nu != -1 && next[nu] * 2 >= i) {
				u = nu;
			}
		}
		if(next[u] * 2 >= i) {
			u = next[u];
		}
		assert(next[u] * 2 < i);
		ans = (ans * (cnt[u] + 1)) % mod;
		dbg(i, u, cnt[u]);
	}
	cout << ans << endl;
}

int main() {
	cin.tie(0)->sync_with_stdio(0);
	cin.exceptions(ios::failbit);
#ifdef LOCAL
	freopen("input.txt", "r", stdin);
#endif
	int t = 1;
	cin >> t;
	for(int _ = 1; _ <= t; _++) {
		dbg(_);
//		cout << "Case #" << _ << ": ";
		solve();
	}
}