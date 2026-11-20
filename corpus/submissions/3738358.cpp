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

set<pair<int, int>> arrange {
	{0, 6},
	{0, 9},
	{2, 3},
	{3, 5},
	{6, 9},
}, valid {
	{0, 8},
	{1, 7},
	{3, 9},
	{5, 6},
	{5, 9},
	{6, 8},
	{8, 9}
};

int cnt[10] = {6, 2, 5, 5, 4, 5, 6, 3, 7, 6};

void solve() {
	string s;
	cin >> s;
	s = s.substr(0, s.find('#'));
	vector<tuple<long, int, int>> digits;
	bool mpos = false;
	long tsum = 0;
	for(int i = 0; i < sz(s);) {
		bool pos = true;
		char c = s[i];
		string cur;
		if(c == '=') {
			mpos = true;
			c = s[++i];
		}
		if(!isdigit(c)) {
			pos = c == '+';
			c = s[++i];
		}
		do {
			cur += c;
		}while(isdigit(c = s[++i]));
		pos ^= mpos;
		long cmul = pos ? 1 : -1, x = stoi(cur);
		tsum += cmul * x;
		int ptr = i;
		if(x == 0) {
			digits.emplace_back(cmul, 0, ptr - 1);
		}
		while(x) {
			digits.emplace_back(cmul, x % 10, --ptr);
			cmul *= 10;
			x /= 10;
		}
	}
	for(auto &[amul, ax, ai]: digits) {
		for(int i = 0; i < 10; i++) {
			int bx = i;
			if(arrange.count(minmax(ax, bx))) {
				long nsum = tsum;
				nsum -= ax * amul;
				nsum += bx * amul;
				if(!nsum) {
					s[ai] = char('0' + bx);
					cout << s << "#" << endl;
					return;
				}
			}
		}
	}
	vector<int> change[10];
	for(auto &[a, b]: valid) {
		change[a].push_back(b);
		change[b].push_back(a);
	}
	for(auto &[amul, ax, ai]: digits) {
		for(auto &[bmul, bx, bi]: digits) {
			if(ai != bi) {
				for(auto &a: change[ax]) {
					for(auto &b: change[bx]) {
						if(cnt[a] + cnt[b] == cnt[ax] + cnt[bx]) {
							long nsum = tsum;
							nsum -= ax * amul;
							nsum -= bx * bmul;
							nsum += a * amul;
							nsum += b * bmul;
							if(!nsum) {
								s[ai] = char('0' + a);
								s[bi] = char('0' + b);
								cout << s << "#" << endl;
								return;
							}
						}
					}
				}
			}
		}
	}
	cout << "No" << endl;
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