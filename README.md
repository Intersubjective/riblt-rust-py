# riblt-rust-py
Python bindings for the [Rateless IBLT Rust library](https://github.com/Intersubjective/riblt-rust).

Maximum supported symbol size is 16384 bytes.
Optimal symbol sizes are: 64, 1024, 4096, 16384.

### How to build
```sh
poetry install
poetry run maturin build
```

### Example
Python program to use the library with SipHash and 1-byte symbols:
```py
import riblt_rust_py as riblt

symbol_size = 1
key = bytes([16, 15, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 2, 1])

enc = riblt.new_encoder_sip(symbol_size, key)

enc.add_symbol(bytes([1]))
enc.add_symbol(bytes([2]))
enc.add_symbol(bytes([4]))

dec = riblt.new_decoder_sip(symbol_size, key)

dec.add_symbol(bytes([1]))
dec.add_symbol(bytes([3]))
dec.add_symbol(bytes([4]))

while True:
  s = enc.produce_next_coded_symbol()
  print("coded: " + str(s.data[0]) + ", " + str(s.hash) + ", " + str(s.count))
  dec.add_coded_symbol(bytes([s.data[0]]), s.hash, s.count)
  dec.try_decode()
  if dec.decoded():
    break

local_symbols  = dec.get_local_symbols()
remote_symbols = dec.get_remote_symbols()

print("local symbol:  " + str(local_symbols[0].data[0]))
print("remote symbol: " + str(remote_symbols[0].data[0]))
```

To run the example:
```sh
poetry run maturin develop
poetry run python example.py
```
