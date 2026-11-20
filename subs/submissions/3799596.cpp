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

void explore(int n, int m) {
	if(n <= 500) {
		int v[n] {};
		for(int i = 0; i < n - 1; i++) {
			modify(i);
			for(int j = i + 1; j < n; j++) {
				int cur = query(j);
				if(v[j] != cur) {
					v[j] = cur;
					report(i, j);
				}
			}
		}
		for(int i = 0; i < n; i++) {
			assert(check(i));
		}
	}else {
		mt19937 cowng(chrono::steady_clock::now().time_since_epoch().count());
		auto rint = [&](int l, int r) -> int {
			return cowng() % (r - l + 1) + l;
		};
		int cur[n] {}, status[n] {};
		auto changed = [&](int i) -> bool {
			int x = query(i);
			if(cur[i] != x) {
				cur[i] = x;
				return true;
			}
			return false;
		};
		for(int i = 0; i < __lg(n); i++) {
			for(int j = 0; j < n / 2; j++) {
				modify(rint(0, n - 1));
			}
			for(int j = 0; j < n; j++) {
				status[j] = (status[j] << 1) | changed(j);
			}
		}
		map<int, vector<int>> vals;
		for(int i = 0; i < n; i++) {
			vals[status[i]].push_back(i);
		}
		for(auto &[k, v]: vals) {
			while(sz(v) != 2) {
				modify(v[0]);
				vector<int> ch;
				for(auto &a: v) {
					if(changed(a)) {
						ch.push_back(a);
					}
				}
				assert(sz(ch) == 2);
				report(ch[0], ch[1]);
			}
			report(v[0], v[1]);
		}
	}
}