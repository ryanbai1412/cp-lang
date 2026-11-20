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
const int maxn = 2e5 + 100;

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

int n;
vector<int> arr;
vector<tuple<int, int, int>> queries;

namespace S1 {
	void answer() {
		vector<int> simu {0, 1};
		for(int i = 0; i < n; i++) {
			int a = arr[i];
			if(a == 0) {
				simu.back()++;
			}else {
				if(simu.back() == 1) {
					simu[sz(simu) - 2]++;
				}else {
					simu.back()--;
					simu.push_back(1);
					simu.push_back(1);
				}
			}
		}
		long num = 1, den = 0;
		for(int i = sz(simu) - 1; i >= 0; i--) {
			long cur = simu[i];
			swap(den, num);
			num = (cur * den + num) % mod;
		}
		cout << num << " " << den << endl;
	}

	void solve() {
		answer();
		for(auto &[t, l, r]: queries) {
			if(t == 2) {
				n++;
			}else if(t == 0) {
				for(int i = l; i <= r; i++) {
					arr[i] ^= 1;
				}
			}else {
				reverse(arr.begin() + l, arr.begin() + r + 1);
			}
			answer();
		}
	}
}

namespace S2 {
	bool wfirst = true;
	long fib[maxn + 1];

	void pcomp() {
		fib[0] = 1;
		fib[1] = 2;
		for(int i = 2; i <= maxn; i++) {
			fib[i] = (fib[i - 2] + fib[i - 1]) % mod;
		}
	}

	bool solvable() {
		for(int i = 0; i < sz(arr) - 1; i++) {
			if(arr[i] == arr[i + 1]) {
				return false;
			}
		}
		return true;
	}

	void answer() {
		if(wfirst) {
			cout << fib[n - 1] << " " << fib[n] << endl;
		}else {
			cout << fib[n] << " " << fib[n - 1] << endl;
		}
	}

	void solve() {
		pcomp();
		wfirst = arr[0] == 0;
		answer();
		for(auto &[t, l, r]: queries) {
			if(t == 2) {
				n++;
			}else if(t == 0) {
				assert(l == 0 && r == n - 1);
				wfirst ^= true;
			}else {
				if((r - l) & 1) {
					assert(l == 0 && r == n - 1);
					wfirst ^= true;
				}
			}
			answer();
		}
	}
}

void solve() {
	int q;
	cin >> n >> q;
	string s;
	cin >> s;
	for(auto &a: s) {
		arr.push_back(a == 'E');
	}
	bool flip = false, reverse = false;
	while(q--) {
		string type;
		cin >> type;
		if(type == "APPEND") {
			char c;
			cin >> c;
			arr.push_back(c == 'E');
			queries.emplace_back(2, 0, 0);
		}else {
			int l, r;
			cin >> l >> r;
			l--;
			r--;
			queries.emplace_back(type == "REVERSE", l, r);
			flip |= type == "FLIP";
			reverse |= type == "REVERSE";
		}
	}
	if(n <= 2000 && sz(queries) <= 2000) {
		S1::solve();
	}else if(S2::solvable()) {
		S2::solve();
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