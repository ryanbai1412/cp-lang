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

const long base = 23, ibase = 739130440, hmod = 1e9 + 7, mod = 1e9 + 7;

struct Hasher {
	int n;
	string s;
	vector<long> psum, ppow, pipow;
	
	explicit Hasher(string s): n(sz(s)), s(s), psum(n + 1), ppow(n + 1), pipow(n + 1) {
		ppow[0] = pipow[0] = 1;
		for(int i = 0; i < n; i++) {
			psum[i + 1] = (psum[i] + ppow[i] * (s[i] - 'a' + 1)) % hmod;
			ppow[i + 1] = (ppow[i] * base) % hmod;
			pipow[i + 1] = (pipow[i] * ibase) % hmod;
		}
	}
	
	long hash(int l, int r) {
		long c = (psum[r + 1] - psum[l] + hmod) % hmod;
		return (c * pipow[l]) % hmod;
	}
};

void solve() {
	string s;
	cin >> s;
	int n = sz(s);
	Hasher h(s);

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
			if(h.hash(0, j - 1) == h.hash(i - j + 1, i)) {
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