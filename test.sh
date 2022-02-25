#!/usr/bin/bash
assert(){
  expected="$1"
  input="$2"

  ./target/debug/9cc "$input" > ./c/tmp.s
  cc -o ./c/tmp ./c/tmp.s
  ./c/tmp
  
  actual="$?"

  if [  "$actual" = "$expected"  ]; then
    echo "$input => $actual"
  else
    echo "$input => $expected, but got $actual"
    exit 1
  fi
}

assert 0 0
assert 42 42
assert 2 "1+1" 
assert 21 "20-9+10"

echo OK