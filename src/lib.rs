use pyo3::prelude::*;
use pyo3::exceptions::*;
use riblt::*;

#[allow(deprecated)]
use std::hash::{SipHasher, Hasher};

const KEY_SIZE : usize = 16;

#[derive(Clone, Copy)]
enum Hash {
  NONE,
  SIP
}

macro_rules! instant {
  ($max_size : expr,
   $Sym      : ident,
   $Hashed   : ident,
   $Coded    : ident,
   $Enc      : ident,
   $Dec      : ident) => {
    #[pyclass]
    #[derive(Clone, Copy)]
    struct $Sym {
      bytes     : [u8; $max_size],
      size      : usize,
      hash_type : Hash,
      hash_key  : [u8; KEY_SIZE],
    }

    #[pyclass]
    struct $Hashed {
      #[pyo3(get, set)]
      pub data : [u8; $max_size],
      #[pyo3(get, set)]
      pub hash : u64,
    }

    #[pyclass]
    struct $Coded {
      #[pyo3(get, set)]
      pub data  : [u8; $max_size],
      #[pyo3(get, set)]
      pub hash  : u64,
      #[pyo3(get, set)]
      pub count : i64,
    }

    #[pyclass]
    struct $Enc {
      enc         : Encoder<$Sym>,
      symbol_size : usize,
      hash_type   : Hash,
      hash_key    : [u8; KEY_SIZE],
    }

    #[pyclass]
    struct $Dec {
      dec         : Decoder<$Sym>,
      symbol_size : usize,
      hash_type   : Hash,
      hash_key    : [u8; KEY_SIZE],
    }

    impl Symbol for $Sym {
      fn zero() -> $Sym {
        return $Sym {
          bytes     : core::array::from_fn(|_| 0),
          size      : 0,
          hash_type : Hash::NONE,
          hash_key  : core::array::from_fn(|_| 0),
        };
      }

      fn xor(&self, other: &$Sym) -> $Sym {
        let (s, t, k) = match self.hash_type {
            Hash::NONE => (other.size, other.hash_type, other.hash_key),
            _          => ( self.size,  self.hash_type,  self.hash_key),
        };
        return $Sym {
          bytes     : core::array::from_fn(|i| self.bytes[i] ^ other.bytes[i]),
          size      : s,
          hash_type : t,
          hash_key  : k,
        };
      }

      #[allow(deprecated)]
      fn hash(&self) -> u64 {
        match self.hash_type {
          Hash::SIP => {
            let k = self.hash_key;
            let mut hasher = SipHasher::new_with_keys(
              u64::from_le_bytes([k[ 0], k[ 1], k[ 2], k[ 3], k[ 4], k[ 5], k[ 6], k[ 7]]),
              u64::from_le_bytes([k[ 8], k[ 9], k[10], k[11], k[12], k[13], k[14], k[15]])
            );
            hasher.write(&self.bytes);
            return hasher.finish();
          },

          _ => {
            return 0;
          },
        }
      }
    }

    #[pymethods]
    impl $Enc {
      fn reset(&mut self) -> PyResult<()> {
        self.enc.reset();
        Ok(())
      }

      fn add_symbol(&mut self, bytes: &[u8]) -> PyResult<()> {
        if bytes.len() > $max_size || bytes.len() != self.symbol_size {
          return Err(PyTypeError::new_err("invalid byte array size"))
        }
        self.enc.add_symbol(&$Sym {
          bytes     : core::array::from_fn(|i| if i < self.symbol_size { bytes[i] } else { 0 }),
          size      : self.symbol_size,
          hash_type : self.hash_type,
          hash_key  : self.hash_key,
        });
        Ok(())
      }

      fn produce_next_coded_symbol(&mut self) -> PyResult<$Coded> {
        let sym = self.enc.produce_next_coded_symbol();
        Ok($Coded {
          data  : sym.symbol.bytes,
          hash  : sym.hash,
          count : sym.count,
        })
      }
    }

    #[pymethods]
    impl $Dec {
      fn reset(&mut self) -> PyResult<()> {
        self.dec.reset();
        Ok(())
      }

      fn add_symbol(&mut self, bytes: &[u8]) -> PyResult<()> {
        if bytes.len() > $max_size || bytes.len() != self.symbol_size {
          return Err(PyTypeError::new_err("invalid byte array size"));
        }
        self.dec.add_symbol(&$Sym {
          bytes     : core::array::from_fn(|i| if i < self.symbol_size { bytes[i] } else { 0 }),
          size      : self.symbol_size,
          hash_type : self.hash_type,
          hash_key  : self.hash_key,
        });
        Ok(())
      }

      fn add_coded_symbol(&mut self, data: &[u8], hash: u64, count: i64) -> PyResult<()> {
        if data.len() != $max_size && data.len() != self.symbol_size {
          return Err(PyTypeError::new_err("invalid data size"));
        }
        self.dec.add_coded_symbol(&CodedSymbol::<$Sym> {
          symbol : $Sym {
            bytes     : core::array::from_fn(|i| if i < self.symbol_size { data[i] } else { 0 }),
            size      : self.symbol_size,
            hash_type : self.hash_type,
            hash_key  : self.hash_key,
          },
          hash  : hash,
          count : count,
        });
        Ok(())
      }

      fn try_decode(&mut self) -> PyResult<()> {
        if self.dec.try_decode().is_err() {
          return Err(PyRuntimeError::new_err("decoding error"));
        }
        Ok(())
      }

      fn decoded(&self) -> PyResult<bool> {
        Ok(self.dec.decoded())
      }

      fn get_remote_symbols(&self) -> PyResult<Vec<$Hashed>> {
        let v       = self.dec.get_remote_symbols();
        let mut pyv = Vec::<$Hashed>::new();
        pyv.reserve_exact(v.len());
        for i in 0..v.len() {
          pyv.push($Hashed {
            data : v[i].symbol.bytes,
            hash : v[i].hash,
          });
        }
        Ok(pyv)
      }

      fn get_local_symbols(&self) -> PyResult<Vec<$Hashed>> {
        let v       = self.dec.get_local_symbols();
        let mut pyv = Vec::<$Hashed>::new();
        pyv.reserve_exact(v.len());
        for i in 0..v.len() {
          pyv.push($Hashed {
            data : v[i].symbol.bytes,
            hash : v[i].hash,
          });
        }
        Ok(pyv)
      }
    }
  };
}

macro_rules! add_types {
  ($module   : ident,
   $Sym      : ident,
   $Hashed   : ident,
   $Coded    : ident,
   $Enc      : ident,
   $Dec      : ident) => {
    $module.add_class::<$Sym>()?;
    $module.add_class::<$Hashed>()?;
    $module.add_class::<$Coded>()?;
    $module.add_class::<$Enc>()?;
    $module.add_class::<$Dec>()?;
  }
}

const SIZE_0   : usize = 64;
const SIZE_1   : usize = 1024;
const SIZE_2   : usize = 4096;
const SIZE_MAX : usize = 16384;

instant!(SIZE_0,   Sym0,   Hashed0,   Coded0,   Enc0,   Dec0  );
instant!(SIZE_1,   Sym1,   Hashed1,   Coded1,   Enc1,   Dec1  );
instant!(SIZE_2,   Sym2,   Hashed2,   Coded2,   Enc2,   Dec2  );
instant!(SIZE_MAX, SymMax, HashedMax, CodedMax, EncMax, DecMax);

#[pyfunction]
fn new_encoder_sip(py: Python, size: usize, key: [u8; KEY_SIZE]) -> PyResult<PyObject> {
  if size <= SIZE_0 {
    return Ok(Enc0 {
      enc         : Encoder::<Sym0>::new(),
      symbol_size : size,
      hash_type   : Hash::SIP,
      hash_key    : key,
    }.into_py(py));
  }
  if size <= SIZE_1 {
    return Ok(Enc1 {
      enc         : Encoder::<Sym1>::new(),
      symbol_size : size,
      hash_type   : Hash::SIP,
      hash_key    : key,
    }.into_py(py));
  }
  if size <= SIZE_2 {
    return Ok(Enc2 {
      enc         : Encoder::<Sym2>::new(),
      symbol_size : size,
      hash_type   : Hash::SIP,
      hash_key    : key,
    }.into_py(py));
  }
  if size <= SIZE_MAX {
    return Ok(EncMax {
      enc         : Encoder::<SymMax>::new(),
      symbol_size : size,
      hash_type   : Hash::SIP,
      hash_key    : key,
    }.into_py(py));
  }
  return Err(PyValueError::new_err("size is too big"));
}

#[pyfunction]
fn new_decoder_sip(py: Python, size: usize, key: [u8; KEY_SIZE]) -> PyResult<PyObject> {
  if size <= SIZE_0 {
    return Ok(Dec0 {
      dec         : Decoder::<Sym0>::new(),
      symbol_size : size,
      hash_type   : Hash::SIP,
      hash_key    : key,
    }.into_py(py));
  }
  if size <= SIZE_1 {
    return Ok(Dec1 {
      dec         : Decoder::<Sym1>::new(),
      symbol_size : size,
      hash_type   : Hash::SIP,
      hash_key    : key,
    }.into_py(py));
  }
  if size <= SIZE_2 {
    return Ok(Dec2 {
      dec         : Decoder::<Sym2>::new(),
      symbol_size : size,
      hash_type   : Hash::SIP,
      hash_key    : key,
    }.into_py(py));
  }
  if size <= SIZE_MAX {
    return Ok(DecMax {
      dec         : Decoder::<SymMax>::new(),
      symbol_size : size,
      hash_type   : Hash::SIP,
      hash_key    : key,
    }.into_py(py));
  }
  return Err(PyValueError::new_err("size is too big"));
}

#[pymodule]
fn riblt_rust_py(_py: Python, m: &PyModule) -> PyResult<()> {
  add_types!(m, Sym0,   Hashed0,   Coded0,   Enc0,   Dec0  );
  add_types!(m, Sym1,   Hashed1,   Coded1,   Enc1,   Dec1  );
  add_types!(m, Sym2,   Hashed2,   Coded2,   Enc2,   Dec2  );
  add_types!(m, SymMax, HashedMax, CodedMax, EncMax, DecMax);
  m.add_function(wrap_pyfunction!(new_encoder_sip, m)?)?;
  m.add_function(wrap_pyfunction!(new_decoder_sip, m)?)?;
  Ok(())
}
