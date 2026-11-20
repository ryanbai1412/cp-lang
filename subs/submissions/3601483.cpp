#include "bits/extc++.h"

using namespace std;

template<class T, class U = less<T>>
using rt = __gnu_pbds::tree<T, __gnu_pbds::null_type, U, __gnu_pbds::rb_tree_tag, __gnu_pbds::tree_order_statistics_node_update>;

template<class T, class U>
void sep(T &out, const string &s, const U &u) {
	out << u;
}

template<class T, class Head, class ...Tail>
void sep(T &out, const string &s, const Head &h, const Tail &...t) {
	out << h << s;
	sep(out, s, t...);
}

#ifdef DEBUG
#define dbg(...)                                                      \
cerr << "L" << __LINE__ << " [" << #__VA_ARGS__ << "]" << ": ";       \
sep(cerr, " | ", __VA_ARGS__);                                        \
cerr << endl
#else
#define cerr if(false) cerr
#define dbg(...) cerr
#endif

//imagine a language where int = long
#define long int64_t

//typing too hard
#define endl "\n"

#define sz(x) int((x).size())

const long mod = 1e9 + 7;

void solve() {
	string s;
	cin >> s;
	int n = sz(s);
	
	long ans = 1;
	int next[n], bound[n];
	next[0] = bound[0] = 0;
	for(int i = 1; i < n; i++) {
		int j = next[i - 1];
		while(j > 0 && s[i] != s[j]) {
			j = next[j - 1];
		}
		if(s[i] == s[j]) {
			j++;
		}
		next[i] = j;
		bound[i] = min((i + 1) / 2, next[i]);
	}
	for(int i = 0; i < n; i++) {
		long cur = 0;
		for(int j = 1; j <= bound[i]; j++) {
			if(s.substr(0, j) == s.substr(i - j + 1, j)) {
				cur++;
			}
		}
		dbg(i, cur);
		ans = (ans * (cur + 1)) % mod;
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