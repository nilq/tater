# tater
A toy programming language build in Rust, inspired by Assembler.

---
[**STATUS**] things are beginning to work now ...
---

Example program;

[test.tat]
```
? here come dat comment
? o shit waddup
put 1024 ? push 1024 bits onto the stack
pop 512  ? pop 512 bits of the stack

extern print; "\nHello world!"
```

`cargo run -- --file test.tat`

stdout:

```
Parsing: 1024
Parsing: 512
Parsing: b00001010010010000110010101101100011011000110111100100000011101110110111101110010011011000110010000100001

Hello world!%  
```

---

Tater is a very basic and useless toy language based on Bit-Assembly by Jellonator.
