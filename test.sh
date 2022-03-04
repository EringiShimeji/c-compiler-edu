#!/usr/bin/bash
assert(){
  expected="$1"
  input="$2"

  ./target/debug/9cc "$input" > ./test/tmp.s
  cc -o ./test/tmp ./test/tmp.s
  ./test/tmp
  
  actual="$?"

  if [  "$actual" = "$expected"  ]; then
    echo "$input => $actual"
  else
    echo "$input => $expected, but got $actual"
    exit 1
  fi
}

mkdir test

assert 0 0
assert 42 42
assert 21 "5+20-4"
assert 41 " 12 + 34 - 5 "
assert 47 '5+6*7'
assert 15 '5*(9-6)'
assert 4 '(3+5)/2'

echo OK