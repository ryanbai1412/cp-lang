// expect: ok, output "0 2"
domain R = 0..3
domain C = 0..3
print((R(2), C(0)).ray(Dir4.down).collect().len, (R(2), C(0)).ray(Dir4.right).collect().len)
