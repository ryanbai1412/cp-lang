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

const long mod = 1e9 + 7;

int parseCommand(const string &s) {
	int xord = 0, set = -1;
	for(int i = sz(s) - 1; i >= 0; i--) {
		if(s[i] == '0' || s[i] == '1') {
			set = s[i] - '0';
			break;
		}else {
			xord ^= 1;
		}
	}
	if(set == -1) {
		return xord;
	}else {
		return 2 + (set ^ xord);
	}
}

// 0: no change
// 1: xor
// 2: set 0
// 3: set 1
void solve() {
	int n, m;
	cin >> n >> m;
	assert(n < 32);
	vector<int> commands[m];
	for(int i = 0; i < m; i++) {
		string s;
		cin >> s;
		for(int j = 0; j < sz(s); j++) {
			int start = j;
			for(; j < sz(s) && s[j] != 'R'; j++);
			commands[i].push_back(parseCommand(s.substr(start, j - start)));
		}
		if(s.back() == 'R') {
			commands[i].push_back(0);
		}
	}
	long ans = 0;
	for(int ax = 1; ax < (1 << n); ax++) {
		long cur = 1;
		for(int i = 0; i < m; i++) {
			bool state[n][4] {};
			for(int j = 0; j < n; j++) {
				if((ax >> j) & 1) {
					if(j + sz(commands[i]) > n) {
						goto loop;
					}else {
						int l = j;
						int r = j + sz(commands[i]);
						for(int k = 0; k < sz(commands[i]); k++) {
							state[l + k][commands[i][k]] = true;
						}
						for(int k = 0; k < l; k++) {
							state[k][0] = true;
						}
						for(int k = r; k < n; k++) {
							state[k][0] = true;
						}
					}
				}
			}
			for(int j = 0; j < n; j++) {
				long cmul = 3;
				if((state[j][0] && state[j][1]) || (state[j][2] && state[j][3])) {
					cmul = 1;
				}else if((state[j][0] || state[j][1]) && (state[j][2] || state[j][3])) {
					cmul = 2;
				}
				cur = (cur * cmul) % mod;
			}
			loop:;
		}
		if(__builtin_popcount(ax) & 1) {
			ans += cur;
		}else {
			ans -= cur;
		}
		ans %= mod;
		if(ans < 0) {
			ans += mod;
		}
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
//	cin >> t;
	for(int _ = 1; _ <= t; _++) {
		dbg(_);
//		cout << "Case #" << _ << ": ";
		solve();
	}
}