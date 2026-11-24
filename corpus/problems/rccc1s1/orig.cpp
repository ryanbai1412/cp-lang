// Reference for Riolku's Mock CCC S1 (no accepted C++ on DMOJ; written for the corpus).
// A word is invalid when some run of letters from "aeiouy" is longer than V or some run of
// consonants (y included) is longer than C.
#include <bits/stdc++.h>
using namespace std;

int main() {
    long long n, c, v;
    string s;
    cin >> n >> c >> v >> s;
    auto vowel = [](char ch) { return string("aeiouy").find(ch) != string::npos; };
    auto consonant = [](char ch) { return ch == 'y' || string("aeiou").find(ch) == string::npos; };
    long long rv = 0, rc = 0;
    bool ok = true;
    for (char ch : s) {
        rv = vowel(ch) ? rv + 1 : 0;
        rc = consonant(ch) ? rc + 1 : 0;
        if (rv > v || rc > c) ok = false;
    }
    cout << (ok ? "YES" : "NO") << "\n";
}
