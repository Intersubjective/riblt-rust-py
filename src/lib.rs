use pyo3::prelude::*;
use pyo3::exceptions::*;
use riblt::*;

#[allow(deprecated)]
use std::hash::{SipHasher, Hasher};

#[pyclass]
#[derive(Clone, Copy)]
struct SymbolSip64 {
  v: [u8; 64],
}

#[pyclass]
struct CodedSymbolSip64 {
  sym: CodedSymbol<SymbolSip64>,
}

#[pyclass]
struct HashedSymbolSip64 {
  #[pyo3(get, set)]
  pub symbol : [u8; 64],
  #[pyo3(get, set)]
  pub hash   : u64,
}

#[pyclass]
struct EncoderSip64 {
  enc: Encoder<SymbolSip64>,
}

#[pyclass]
struct DecoderSip64 {
  dec: Decoder<SymbolSip64>,
}

impl Symbol for SymbolSip64 {
  fn zero() -> SymbolSip64 {
    return SymbolSip64 {
      v: core::array::from_fn(|_| 0),
    };
  }

  fn xor(&self, other: &SymbolSip64) -> SymbolSip64 {
    return SymbolSip64 {
      v: core::array::from_fn(|i| self.v[i] ^ other.v[i]),
    };
  }

  #[allow(deprecated)]
  fn hash(&self) -> u64 {
    let mut hasher = SipHasher::new_with_keys(567, 890);
    hasher.write(&self.v);
    return hasher.finish();
  }
}

#[pymethods]
impl EncoderSip64 {
  fn reset(&mut self) -> PyResult<()> {
    self.enc.reset();
    Ok(())
  }

  fn add_symbol(&mut self, bytes: &[u8]) -> PyResult<()> {
    if bytes.len() != 64 {
      return Err(PyTypeError::new_err("invalid bytearray size"))
    }
    self.enc.add_symbol(&SymbolSip64 {
      v: core::array::from_fn(|i| bytes[i]),
    });
    Ok(())
  }

  fn produce_next_coded_symbol(&mut self) -> PyResult<CodedSymbolSip64> {
    Ok(CodedSymbolSip64 {
      sym: self.enc.produce_next_coded_symbol(),
    })
  }
}

#[pymethods]
impl DecoderSip64 {
  fn reset(&mut self) -> PyResult<()> {
    self.dec.reset();
    Ok(())
  }

  fn add_symbol(&mut self, bytes: &[u8]) -> PyResult<()> {
    if bytes.len() != 64 {
      return Err(PyTypeError::new_err("invalid byte array size"))
    }
    self.dec.add_symbol(&SymbolSip64 {
      v: core::array::from_fn(|i| bytes[i]),
    });
    Ok(())
  }

  fn add_coded_symbol(&mut self, sym: &CodedSymbolSip64) -> PyResult<()> {
    self.dec.add_coded_symbol(&sym.sym);
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

  fn get_remote_symbols(&self) -> PyResult<Vec<HashedSymbolSip64>> {
    let v       = self.dec.get_remote_symbols();
    let mut pyv = Vec::<HashedSymbolSip64>::new();
    pyv.reserve_exact(v.len());
    for i in 0..v.len() {
      pyv.push(HashedSymbolSip64 {
        symbol : v[i].symbol.v,
        hash   : v[i].hash,
      });
    }
    Ok(pyv)
  }

  fn get_local_symbols(&self) -> PyResult<Vec<HashedSymbolSip64>> {
    let v       = self.dec.get_local_symbols();
    let mut pyv = Vec::<HashedSymbolSip64>::new();
    pyv.reserve_exact(v.len());
    for i in 0..v.len() {
      pyv.push(HashedSymbolSip64 {
        symbol : v[i].symbol.v,
        hash   : v[i].hash,
      });
    }
    Ok(pyv)
  }
}

#[pyfunction]
fn new_encoder_sip_64() -> PyResult<EncoderSip64> {
  return Ok(EncoderSip64 {
    enc: Encoder::<SymbolSip64>::new(),
  });
}

#[pyfunction]
fn new_decoder_sip_64() -> PyResult<DecoderSip64> {
  return Ok(DecoderSip64 {
    dec: Decoder::<SymbolSip64>::new(),
  });
}  

#[pymodule]
fn riblt_rust_py(_py: Python, m: &PyModule) -> PyResult<()> {
  m.add_class::<SymbolSip64>()?;
  m.add_class::<CodedSymbolSip64>()?;
  m.add_class::<HashedSymbolSip64>()?;
  m.add_class::<EncoderSip64>()?;
  m.add_class::<DecoderSip64>()?;
  m.add_function(wrap_pyfunction!(new_encoder_sip_64, m)?)?;
  m.add_function(wrap_pyfunction!(new_decoder_sip_64, m)?)?;
  Ok(())
}
