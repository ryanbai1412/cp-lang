#include "bits/extc++.h"

using namespace std;

template <typename T, typename... U>
void dbgh(const T& t, const U&... u) {
    cerr << t;
    ((cerr << " | " << u), ...);
    cerr << endl;
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
#define sz(x) int(std::size(x))

string print() {
    return "string gen_printer(string s) {\n    string code = R\"FUNC(#include \"bits/extc++.h\"\n\nusing namespace std;\n\ntemplate <typename T, typename... U>\nvoid dbgh(const T& t, const U&... u) {\n    cerr << t;\n    ((cerr << \" | \" << u), ...);\n    cerr << endl;\n}\n\n#ifdef DEBUG\n#define dbg(...)                                           \\\n    cerr << \"L\" << __LINE__ << \" [\" << #__VA_ARGS__ << \"]\" \\\n         << \": \";                                          \\\n    dbgh(__VA_ARGS__)\n#else\n#define cerr   \\\n    if (false) \\\n    cerr\n#define dbg(...)\n#endif\n\n#define endl \"\\n\"\n#define long int64_t\n#define sz(x) int(std::size(x))\n\nstring print() {\n    return \"%S%\";\n}\n\n)FUNC\";\n\n    string escaped = \"\";\n    for (auto& a : s) {\n        if (a == '\\\\') {\n            escaped += \"\\\\\\\\\";\n        } else if (a == '\"') {\n            escaped += \"\\\\\\\"\";\n        } else if (a == '\\n') {\n            escaped += \"\\\\n\";\n        } else {\n            escaped += a;\n        }\n    }\n\n    code.replace(code.find(\"%S%\"), 3, escaped);\n\n    return code;\n}\n\nvoid solve() {\n    cout << gen_printer(print()) << print();\n}\n\nint main() {\n    cin.tie(nullptr);\n    cin.exceptions(ios::failbit);\n    ios_base::sync_with_stdio(false);\n    solve();\n}\n";
}

string gen_printer(string s) {
    string code = R"FUNC(#include "bits/extc++.h"

using namespace std;

template <typename T, typename... U>
void dbgh(const T& t, const U&... u) {
    cerr << t;
    ((cerr << " | " << u), ...);
    cerr << endl;
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
#define sz(x) int(std::size(x))

string print() {
    return "%S%";
}

)FUNC";

    string escaped = "";
    for (auto& a : s) {
        if (a == '\\') {
            escaped += "\\\\";
        } else if (a == '"') {
            escaped += "\\\"";
        } else if (a == '\n') {
            escaped += "\\n";
        } else {
            escaped += a;
        }
    }

    code.replace(code.find("%S%"), 3, escaped);

    return code;
}

void solve() {
    cout << gen_printer(print()) << print();
}

int main() {
    cin.tie(nullptr);
    cin.exceptions(ios::failbit);
    ios_base::sync_with_stdio(false);
    solve();
}