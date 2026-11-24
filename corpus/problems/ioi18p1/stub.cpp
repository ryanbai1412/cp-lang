// The judge's side for the accepted C++: reads N from interactor.py, calls
// guess_sequence, and speaks `press p` / its score and `guess S` on stdio.
#include <iostream>
#include <string>

std::string guess_sequence(int N);

int press(std::string p) {
    std::cout << "press " << p << std::endl;
    int coins;
    std::cin >> coins;
    return coins;
}

int main() {
    int n;
    std::cin >> n;
    std::string s = guess_sequence(n);
    std::cout << "guess " << s << std::endl;
}
