# riblt-rust-py
Python bindings for the [Rateless IBLT Rust library](https://github.com/Intersubjective/riblt-rust).

### How to build
```sh
python3 -m venv .env
source .env/bin/activate
pip install maturin
maturin build
```

### Example
Python program to use the library with Sip hash and 64-byte symbols:
```py
import riblt_rust_py as riblt

zeros = [0] * 63

sym0 = bytes([1] + zeros)
sym1 = bytes([2] + zeros)
sym2 = bytes([3] + zeros)
sym3 = bytes([4] + zeros)

enc = riblt.new_encoder_sip_64()

enc.add_symbol(sym0)
enc.add_symbol(sym1)
enc.add_symbol(sym3)

dec = riblt.new_decoder_sip_64()

dec.add_symbol(sym0)
dec.add_symbol(sym2)
dec.add_symbol(sym3)

while True:
  s = enc.produce_next_coded_symbol()
  dec.add_coded_symbol(s)
  dec.try_decode()
  if dec.decoded():
    break

local_symbols  = dec.get_local_symbols()
remote_symbols = dec.get_remote_symbols()

print("local symbol:  " + str(local_symbols[0].symbol[0]))
print("remote symbol: " + str(remote_symbols[0].symbol[0]))
```
