//! The raw boundary to the game: the three host imports and the guest memory helpers the host
//! uses. On a non-wasm target (so the crate builds for tests and `cargo clippy` on the host) the
//! imports are stubs that panic - a Rust plugin only runs inside openOMSI's WebAssembly host.

/// Packs a pointer and a length the way the ABI does: `(ptr << 32) | len`.
#[inline]
fn unpack(packed: i64) -> (i32, i32) {
    ((packed >> 32) as i32, (packed & 0xffff_ffff) as i32)
}

#[cfg(all(target_arch = "wasm32", target_os = "unknown"))]
#[link(wasm_import_module = "openomsi")]
unsafe extern "C" {
    fn call(name_ptr: i32, name_len: i32, args_ptr: i32, args_len: i32) -> i64;
    fn last_error() -> i64;
    fn log(level: i32, ptr: i32, len: i32);
}

#[cfg(not(all(target_arch = "wasm32", target_os = "unknown")))]
#[allow(unused_variables)]
mod stub {
    pub unsafe fn call(name_ptr: i32, name_len: i32, args_ptr: i32, args_len: i32) -> i64 {
        panic!("openomsi plugin host call used outside the game (build for wasm32-unknown-unknown)")
    }
    pub unsafe fn last_error() -> i64 {
        0
    }
    pub unsafe fn log(level: i32, ptr: i32, len: i32) {}
}
#[cfg(not(all(target_arch = "wasm32", target_os = "unknown")))]
use stub::{call, last_error, log};

/// Allocates `len` bytes for the host to write into and hands back the pointer. The host (or this
/// crate) frees it later with [`free`].
pub fn alloc(len: i32) -> i32 {
    if len < 0 {
        return 0;
    }
    let buf = vec![0u8; len as usize].into_boxed_slice();
    Box::into_raw(buf) as *mut u8 as i32
}

/// Frees `len` bytes that came from [`alloc`].
///
/// # Safety
/// `ptr` and `len` must be exactly a pair returned by [`alloc`] and not yet freed.
pub unsafe fn free(ptr: i32, len: i32) {
    if ptr == 0 || len < 0 {
        return;
    }
    // Reconstruct the exact Box<[u8]> that `alloc` leaked, then drop it.
    let slice = std::ptr::slice_from_raw_parts_mut(ptr as *mut u8, len as usize);
    drop(unsafe { Box::from_raw(slice) });
}

/// Copies `len` bytes from guest memory at `ptr`, then frees that buffer (the receiver owns what
/// it is handed, by the ABI's convention).
///
/// # Safety
/// `ptr`/`len` must name a live buffer from [`alloc`].
unsafe fn take(ptr: i32, len: i32) -> Vec<u8> {
    if ptr == 0 || len <= 0 {
        return Vec::new();
    }
    let src = std::ptr::slice_from_raw_parts(ptr as *const u8, len as usize);
    let bytes = unsafe { &*src }.to_vec();
    unsafe { free(ptr, len) };
    bytes
}

/// The message of the last failed [`host_call`].
fn read_last_error() -> String {
    let packed = unsafe { last_error() };
    if packed <= 0 {
        return "unknown error".to_string();
    }
    let (ptr, len) = unpack(packed);
    String::from_utf8_lossy(&unsafe { take(ptr, len) }).into_owned()
}

/// Calls a registry function by name with JSON-encoded args; returns the JSON-encoded result
/// bytes, or the host's error message.
pub fn host_call(name: &str, args_json: &[u8]) -> Result<Vec<u8>, String> {
    let packed =
        unsafe { call(name.as_ptr() as i32, name.len() as i32, args_json.as_ptr() as i32, args_json.len() as i32) };
    if packed < 0 {
        return Err(read_last_error());
    }
    let (ptr, len) = unpack(packed);
    Ok(unsafe { take(ptr, len) })
}

/// Writes a line into `game.log` (levels: 0 debug, 1 info, 2 warn, 3 error).
pub fn host_log(level: i32, msg: &str) {
    unsafe { log(level, msg.as_ptr() as i32, msg.len() as i32) }
}

/// Reads a JSON argument buffer the host passed to `oop_callback`, freeing it afterwards.
///
/// # Safety
/// `ptr`/`len` must be the arguments the host passed to `oop_callback`.
pub unsafe fn take_callback_args(ptr: i32, len: i32) -> Vec<u8> {
    unsafe { take(ptr, len) }
}
