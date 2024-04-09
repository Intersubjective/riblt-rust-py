use pyo3::prelude::*;
use pyo3::exceptions::*;
use riblt::*;

#[allow(deprecated)]
use std::hash::{SipHasher, Hasher};

const MAX_SIZE : usize = 64;

#[derive(Clone, Copy)]
enum Hash {
  NONE,
  SIP
}

#[pyclass]
#[derive(Clone, Copy)]
struct PySymbol {
  bytes     : [u8; MAX_SIZE],
  size      : usize,
  hash_type : Hash,
  hash_keys : (u64, u64),
}

#[pyclass]
struct PyHashedSymbol {
  #[pyo3(get, set)]
  pub data : [u8; MAX_SIZE],
  #[pyo3(get, set)]
  pub hash : u64,
}

#[pyclass]
struct PyCodedSymbol {
  #[pyo3(get, set)]
  pub data  : [u8; MAX_SIZE],
  #[pyo3(get, set)]
  pub hash  : u64,
  #[pyo3(get, set)]
  pub count : i64,
}

#[pyclass]
struct PyEncoder {
  enc         : Encoder<PySymbol>,
  symbol_size : usize,
  hash_type   : Hash,
  hash_keys   : (u64, u64),
}

#[pyclass]
struct PyDecoder {
  dec         : Decoder<PySymbol>,
  symbol_size : usize,
  hash_type   : Hash,
  hash_keys   : (u64, u64),
}

impl Symbol for PySymbol {
  fn zero() -> PySymbol {
    return PySymbol {
      bytes     : core::array::from_fn(|_| 0),
      size      : 0,
      hash_type : Hash::NONE,
      hash_keys : (0, 0),
    };
  }

  fn xor(&self, other: &PySymbol) -> PySymbol {
    let (s, t, k0, k1) = match self.hash_type {
        Hash::NONE => (other.size, other.hash_type, other.hash_keys.0, other.hash_keys.1),
        _          => ( self.size,  self.hash_type,  self.hash_keys.0,  self.hash_keys.1),
    };
    return PySymbol {
      bytes     : core::array::from_fn(|i| self.bytes[i] ^ other.bytes[i]),
      size      : s,
      hash_type : t,
      hash_keys : (k0, k1),
    };
  }

  #[allow(deprecated)]
  fn hash(&self) -> u64 {
    match self.hash_type {
      Hash::SIP => {
        let mut hasher = SipHasher::new_with_keys(self.hash_keys.0, self.hash_keys.1);
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
impl PyEncoder {
  fn reset(&mut self) -> PyResult<()> {
    self.enc.reset();
    Ok(())
  }

  fn add_symbol(&mut self, bytes: &[u8]) -> PyResult<()> {
    if bytes.len() > MAX_SIZE || bytes.len() != self.symbol_size {
      return Err(PyTypeError::new_err("invalid bytearray size"))
    }
    self.enc.add_symbol(&PySymbol {
      bytes     : core::array::from_fn(|i| if i < self.symbol_size { bytes[i] } else { 0 }),
      size      : self.symbol_size,
      hash_type : self.hash_type,
      hash_keys : self.hash_keys,
    });
    Ok(())
  }

  fn produce_next_coded_symbol(&mut self) -> PyResult<PyCodedSymbol> {
    let sym = self.enc.produce_next_coded_symbol();
    Ok(PyCodedSymbol {
      data  : sym.symbol.bytes,
      hash  : sym.hash,
      count : sym.count,
    })
  }
}

#[pymethods]
impl PyDecoder {
  fn reset(&mut self) -> PyResult<()> {
    self.dec.reset();
    Ok(())
  }

  fn add_symbol(&mut self, bytes: &[u8]) -> PyResult<()> {
    if bytes.len() > MAX_SIZE || bytes.len() != self.symbol_size {
      return Err(PyTypeError::new_err("invalid byte array size"))
    }
    self.dec.add_symbol(&PySymbol {
      bytes     : core::array::from_fn(|i| if i < self.symbol_size { bytes[i] } else { 0 }),
      size      : self.symbol_size,
      hash_type : self.hash_type,
      hash_keys : self.hash_keys,
    });
    Ok(())
  }

  fn add_coded_symbol(&mut self, sym: &PyCodedSymbol) -> PyResult<()> {
    self.dec.add_coded_symbol(&CodedSymbol::<PySymbol> {
      symbol : PySymbol {
        bytes     : sym.data,
        size      : self.symbol_size,
        hash_type : self.hash_type,
        hash_keys : self.hash_keys,
      },
      hash  : sym.hash,
      count : sym.count,
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

  fn get_remote_symbols(&self) -> PyResult<Vec<PyHashedSymbol>> {
    let v       = self.dec.get_remote_symbols();
    let mut pyv = Vec::<PyHashedSymbol>::new();
    pyv.reserve_exact(v.len());
    for i in 0..v.len() {
      pyv.push(PyHashedSymbol {
        data : v[i].symbol.bytes,
        hash : v[i].hash,
      });
    }
    Ok(pyv)
  }

  fn get_local_symbols(&self) -> PyResult<Vec<PyHashedSymbol>> {
    let v       = self.dec.get_local_symbols();
    let mut pyv = Vec::<PyHashedSymbol>::new();
    pyv.reserve_exact(v.len());
    for i in 0..v.len() {
      pyv.push(PyHashedSymbol {
        data : v[i].symbol.bytes,
        hash : v[i].hash,
      });
    }
    Ok(pyv)
  }
}

#[pyfunction]
fn new_encoder_sip(size: usize, key_0: u64, key_1: u64) -> PyResult<PyEncoder> {
  return Ok(PyEncoder {
    enc         : Encoder::<PySymbol>::new(),
    symbol_size : size,
    hash_type   : Hash::SIP,
    hash_keys   : (key_0, key_1),
  });
}

#[pyfunction]
fn new_decoder_sip(size: usize, key_0: u64, key_1: u64) -> PyResult<PyDecoder> {
  return Ok(PyDecoder {
    dec         : Decoder::<PySymbol>::new(),
    symbol_size : size,
    hash_type   : Hash::SIP,
    hash_keys   : (key_0, key_1),
  });
}  

#[pymodule]
fn riblt_rust_py(_py: Python, m: &PyModule) -> PyResult<()> {
  m.add_class::<PySymbol>()?;
  m.add_class::<PyHashedSymbol>()?;
  m.add_class::<PyCodedSymbol>()?;
  m.add_class::<PyEncoder>()?;
  m.add_class::<PyDecoder>()?;
  m.add_function(wrap_pyfunction!(new_encoder_sip, m)?)?;
  m.add_function(wrap_pyfunction!(new_decoder_sip, m)?)?;
  Ok(())
}
