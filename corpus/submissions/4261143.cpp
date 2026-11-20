#pragma GCC optimize("Ofast,unroll-loops")
#pragma GCC target("avx2")

#include "bits/extc++.h"
#include "word.h"

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
#define dbg(...) 1412
#endif

#define endl "\n"
#define long int64_t
#define sz(x) int((x).size())

using str = array<char, 5>;
using res = array<bool, 5>;

vector<str> dict, dcands, cand;

void reset(char initial_letter) {
	cand.clear();
	for (auto& a : dcands) {
		if (a[0] == initial_letter) {
			cand.push_back(a);
		}
	}
}

pair<res, res> evaluate(const str& guess, const str& ans) {
	res gold {}, silver {};
	for (int i = 0; i < 5; i++) {
		gold[i] = guess[i] == ans[i];
	}
	for (int i = 0; i < 5; i++) {
		if (!gold[i]) {
			for (int j = 0; j < 5; j++) {
				if (!gold[j] && guess[i] == ans[j]) {
					silver[i] = true;
				}
			}
		}
	}
	return {gold, silver};
}

int hsh(const pair<res, res> &result) {
	int ans = 0;
	auto go = [&](const res &r) {
		for (auto& a : r) {
			ans = ans * 2 + a;
		}
	};
	go(result.first);
	go(result.second);
	return ans;
}

const char* guess(int /*num_testcase*/,
				  int remaining_guesses,
				  char initial_letter,
				  bool* gold,
				  bool* silver) {
	static str last_guess;
	static mt19937 cowng(chrono::steady_clock::now().time_since_epoch().count());
	shuffle(begin(dict), end(dict), cowng);
	if (remaining_guesses == 5) {
		reset(initial_letter);
		last_guess = {'l', 'e', 'a', 'n', 't'};
		return last_guess.begin();
	} else if (remaining_guesses == 0) {
		return "paper";
	} else {
		vector<str> filterd;
		for (auto& a : cand) {
			auto [cg, cs] = evaluate(last_guess, a);
			if (equal(begin(cg), end(cg), gold) && equal(begin(cs), end(cs), silver)) {
				filterd.push_back(a);
			}
		}
		assert(sz(filterd));
		cand = filterd;
	}
	if (remaining_guesses == 1) {
		last_guess = cand[cowng() % sz(cand)];
		return last_guess.begin();
	}
	if (sz(cand) == 1) {
		last_guess = cand[0];
		return last_guess.begin();
	}
	pair<int, str> opt {1e9, {'a', 'a', 'a', 'a', 'a'}};
	for (auto& a : dict) {
		int cnts[1024] {};
		for (auto& b : cand) {
			cnts[hsh(evaluate(a, b))]++;
		}
		pair<int, str> cv {*max_element(begin(cnts), end(cnts)), a};
		opt = min(opt, cv);
	}
	last_guess = opt.second;
	return last_guess.begin();
}
void init(int num_scramble, const char* scramble) {
	for (int i = 0; i < num_scramble; i++) {
		str s;
		copy(scramble + i * 5, scramble + i * 5 + 5, s.begin());
		dict.push_back(s);
	}
	static const int NUM_SCRAMBLE = 8869;
	srand(0);
	for (int i = 0; i < 1000; i++) {
		int answer = rand() % NUM_SCRAMBLE;
		dcands.push_back(dict[answer]);
	}
	srand(0);
}