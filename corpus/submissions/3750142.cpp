#pragma GCC optimize("Ofast,unroll-loops,no-stack-protector")
#pragma GCC target("mmx,sse,sse2,sse3,ssse3,sse4,sse4a,sse4.1,sse4.2,avx,avx2,fma,fma4,popcnt,abm,bmi,bmi2")

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

namespace IO {
	const int maxb = 20 << 20;

	int bind = 0;
	char buf[maxb];

	void init() {
		fread_unlocked(buf, 1, maxb, stdin);
	}

	inline int nint() {
		char c = buf[bind++];
		bool neg = false;
		if(c == '-') {
			neg = true;
			c = buf[bind++];
		}
		int ret = 0;
		while(c >= '0') {
			ret = ret * 10 + (c - '0');
			c = buf[bind++];
		}
		if(neg) {
			return -ret;
		}
		return ret;
	}

	inline int ntype() {
		int start = bind;
		while(buf[bind++] >= '-');
		switch(buf[start]) {
			case 'I':
				return 1;
			case 'D':
				return 2;
			case 'M':
				return bind - start == 10 ? 3 : 6;
			case 'R':
				return 4;
			case 'G':
				return 5;
			default:
				assert(false);
		}
	}
}

using IO::nint, IO::ntype;

void solve() {
	int n = nint(), q = nint();
	vector<int> arr;
	arr.reserve(1e5);
	for(int i = 0; i < n; i++) {
		arr.push_back(nint());
	}
	while(q--) {
		int t = ntype();
		if(t == 1) {
			int ind = nint(), m = nint();
			int carr[m];
			for(int i = 0; i < m; i++) {
				carr[i] = nint();
			}
			arr.insert(arr.begin() + ind, carr, carr + m);
		}else if(t == 6) {
			int mx = -1e9, ans = 0, sum = 0;
			for(auto &a: arr) {
				mx = max(mx, a);
				ans = max(ans, sum = max(sum + a, 0));
			}
			if(mx < 0) {
				cout << mx << endl;
			}else {
				cout << ans << endl;
			}
		}else {
			auto l = arr.begin() + (nint() - 1);
			auto r = l + nint();
			if(t == 2) {
				arr.erase(l, r);
			}else if(t == 3) {
				fill(l, r, nint());
			}else if(t == 4) {
				reverse(l, r);
			}else if(t == 5) {
				cout << accumulate(l, r, 0) << endl;
			}
		}
	}
}

int main() {
	cin.tie(0)->sync_with_stdio(0);
	cin.exceptions(ios::failbit);
#ifdef LOCAL
	freopen("input.txt", "r", stdin);
#endif
	IO::init();
	int t = 1;
//	cin >> t;
	for(int _ = 1; _ <= t; _++) {
		dbg(_);
//		cout << "Case #" << _ << ": ";
		solve();
	}
}