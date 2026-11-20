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

const long mod = 998244353;

long bpow(long base, long exp) {
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

long inv(long x) {
	return bpow(x, mod - 2);
}

long deter(const vector<vector<long>> &arr) {
	int n = sz(arr);
	vector<int> perm(n);
	iota(begin(perm), end(perm), 0);
	long ans = 0;
	do {
		int inv = 0;
		for(int i = 0; i < n; i++) {
			if(!arr[i][perm[i]]) {
				goto loop1;
			}
		}
		for(int i = 0; i < n; i++) {
			for(int j = i + 1; j < n; j++) {
				inv += perm[i] > perm[j];
			}
		}
		if(inv & 1) {
			ans--;
		}else {
			ans++;
		}
		loop1:;
	}while(next_permutation(begin(perm), end(perm)));
	return (ans + mod) % mod;
}

int check(const vector<vector<long>> &arr) {
	int n = sz(arr);
	vector<int> perm(n);
	iota(begin(perm), end(perm), 0);
	do {
		int inv = 0;
		for(int i = 0; i < n; i++) {
			if(!arr[i][perm[i]]) {
				goto loop1;
			}
		}
		for(int i = 0; i < n; i++) {
			for(int j = i + 1; j < n; j++) {
				inv += perm[i] > perm[j];
			}
		}
		if(inv & 1) {
			return mod - 1;
		}else {
			return 1;
		}
		loop1:;
	}while(next_permutation(begin(perm), end(perm)));
	return 0;
}

void solve() {
	int k;
	cin >> k;
	int n;
	cin >> n;
	for(int i = 0; i < k - 1; i++) {
		int x;
		cin >> x;
		assert(x == n);
	}
	int ms[k - 1];
	for(int i = 0; i < k - 1; i++) {
		cin >> ms[i];
	}
	long ans = 1;
	for(auto &m: ms) {
		vector<vector<long>> arr(n, vector<long>(n));
		for(int i = 0; i < m; i++) {
			int u, v;
			cin >> u >> v;
			u--;
			v--;
			arr[u][v] = 1;
		}
		ans = (ans * deter(arr)) % mod;
		if(!ans) {
			cout << 0 << endl;
			return;
		}
	}
	if(ans < 0) {
		ans += mod;
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