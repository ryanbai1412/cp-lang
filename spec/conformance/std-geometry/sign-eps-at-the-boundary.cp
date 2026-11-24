// expect: ok, output "0 0 1"
print(sign_eps(0.0), sign_eps(-1e-9), sign_eps(0.5, eps: 0.25))
