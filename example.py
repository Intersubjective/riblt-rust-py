import riblt_rust_py as riblt

def example():
  symbol_size = 1
  key_0       = 123
  key_1       = 456

  enc = riblt.new_encoder_sip(symbol_size, key_0, key_1)

  enc.add_symbol(bytes([1]))
  enc.add_symbol(bytes([2]))
  enc.add_symbol(bytes([4]))

  dec = riblt.new_decoder_sip(symbol_size, key_0, key_1)

  dec.add_symbol(bytes([1]))
  dec.add_symbol(bytes([3]))
  dec.add_symbol(bytes([4]))

  while True:
    s = enc.produce_next_coded_symbol()
    print("coded: " + str(s.data[0]) + ", " + str(s.hash) + ", " + str(s.count))
    dec.add_coded_symbol(s)
    dec.try_decode()
    if dec.decoded():
      break

  local_symbols  = dec.get_local_symbols()
  remote_symbols = dec.get_remote_symbols()

  print("local symbol:  " + str(local_symbols[0].data[0]))
  print("remote symbol: " + str(remote_symbols[0].data[0]))

example()
