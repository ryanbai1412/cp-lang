#include "bits/extc++.h"

using namespace std;

template <typename T>
void dbgh(const T& t) {
	cerr << t << endl;
}

template <typename T, typename... U>
void dbgh(const T& t, const U&... u) {
	cerr << t << " | ";
	dbgh(u...);
}

#ifdef DEBUG
#define dbg(...)                                           \
	cerr << "L" << __LINE__ << " [" << #__VA_ARGS__ << "]" \
		 << ": ";                                          \
	dbgh(__VA_ARGS__)
#else
#define cerr   \
	if (false) \
	cerr
#define dbg(...)
#endif

#define endl "\n"
#define long int64_t
#define sz(x) int((x).size())

map<string, string> vals;

bool isymbol(char c) {
	return isalnum(c) || c == '_';
}

string expand(string s, const set<string>& vis) {
	string ans;
	for (int i = 0; i < sz(s);) {
		for (; i < sz(s) && !isymbol(s[i]); i++) {
			ans += s[i];
		}
		if (i == sz(s)) {
			continue;
		}
		int start = i;
		for (; i < sz(s) && isymbol(s[i]); i++)
			;
		string cur = s.substr(start, i - start);
		if (!vis.count(cur)) {
			auto it = vals.find(cur);
			if (it != vals.end()) {
				auto tmp = vis;
				tmp.insert(cur);
				cur = expand(it->second, tmp);
			}
		}
		ans += cur;
	}
	return ans;
}

void solve() {
	int n;
	{
		string s;
		getline(cin, s);
		n = stoi(s);
	}
	for (int i = 0; i < n; i++) {
		string s;
		getline(cin, s);
		if (s[0] == '#') {
			stringstream ss(s);
			ss.exceptions(ios::failbit);
			string type;
			ss >> type;
			if (type == "#define") {
				string a, b;
				ss >> a >> b;
				for (auto& c : a) {
					assert(isymbol(c));
				}
				for (auto& c : b) {
					assert(isymbol(c));
				}
				vals[a] = b;
			} else if (type == "#undef") {
				string a;
				ss >> a;
				auto it = vals.find(a);
//				assert(it != vals.end());
				vals.erase(it);
			} else {
				assert(false);
			}
			cout << endl;
		} else {
			cout << expand(s, {}) << endl;
		}
	}
}

int main() {
	cin.tie(nullptr);
	ios_base::sync_with_stdio(false);
	cin.exceptions(ios::failbit);
	solve();
}