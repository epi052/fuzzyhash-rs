mod blockhash;
pub mod compare;
pub mod constants;
mod hasher;
mod roll;

pub use hasher::Hasher;
use std::ffi::{CStr, CString};
use std::io::Read;
use std::path::Path;

/// Returns the fuzzy hash of arbitrary data
///
/// # Arguments
/// * `buf` - a Vec<u8> containing the data to hash
///
/// # Example
/// ```
/// use fuzzyhash::{hash_buffer};
/// let data = "this is our test data!".to_string().as_bytes().to_vec();
/// println!("Fuzzy Hash: {}", hash_buffer(data));
/// ```
pub fn hash_buffer(buf: Vec<u8>) -> String {
    hash_array(&buf, buf.len())
}

/// Returns the fuzzy hash of arbitrary data.
///
/// # Arguments
/// * `buf` - a &[u8] containing the data to hash
///
/// # Example
/// ```
/// use fuzzyhash::{hash_array};
/// let data = "this is our test data!".to_string();
/// println!("Fuzzy Hash: {}", hash_array(data.as_bytes(), data.len()));
/// ```
pub fn hash_array(buf: &[u8], length: usize) -> String {
    let mut hasher = Hasher::new();
    hasher.update(buf, length);
    hasher.digest(constants::Modes::None)
}

/// Returns the fuzzy hash of arbitrary data. This method provides better FFI compatibility.
///
/// # Arguments
/// * `buf` - a pointer to the array containing the data to hash
/// * `length` - length of buf
/// # Example
/// ```
/// use fuzzyhash::{hash_buffer_raw};
/// use std::ffi::CString;
///
/// let data = "this is our test data!".to_string();
/// let hash = unsafe { CString::from_raw(hash_buffer_raw(data.as_bytes().as_ptr(), data.len())) };
/// println!("Fuzzy Hash: {}", hash.into_string().unwrap());
/// ```
#[no_mangle]
pub extern "C" fn hash_buffer_raw(buf: *const u8, length: usize) -> *mut i8 {
    let data = unsafe { std::slice::from_raw_parts(buf, length) };
    let s = CString::new(hash_array(data, length)).unwrap();
    s.into_raw()
}

/// FFI Compatible fuzzy hash comparisons.
///
/// # Arguments
/// * `first` - a C style fuzzy hash string
/// * `second` - a C style fuzzy hash string
///
/// # Example
/// ```
/// use fuzzyhash::{compare_strings_raw};
/// use std::ffi::CString;
///
/// let first = CString::new("this is our test data!").unwrap();
/// let second = CString::new("this is my test data!").unwrap();
/// println!("Fuzzy Hash: {}", compare_strings_raw(first.into_raw(), second.into_raw()));
/// ```
#[no_mangle]
pub extern "C" fn compare_strings_raw(first: *const i8, second: *const i8) -> u32 {
    let f = unsafe { CStr::from_ptr(first) }
        .to_string_lossy()
        .into_owned();
    let s = unsafe { CStr::from_ptr(second) }
        .to_string_lossy()
        .into_owned();

    let x = compare::strs(&f, &s);
    x
}

pub fn hash_file<P: AsRef<Path>>(path: P) -> Result<String, std::io::Error> {
    let mut file = std::fs::File::open(path.as_ref())?;
    let mut hasher = Hasher::new();
    loop {
        let mut buffer = [0; 1024];
        let len = file.read(&mut buffer)?;
        hasher.update(&buffer, len);

        if len < 1024 {
            break;
        }
    }
    Ok(hasher.digest(constants::Modes::None))
}
