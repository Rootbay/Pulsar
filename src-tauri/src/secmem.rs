use std::alloc::{alloc, dealloc, Layout};
use std::ptr::NonNull;
use subtle::ConstantTimeEq;
use zeroize::Zeroize;

/// A page-aligned heap buffer that is locked in physical RAM using OS APIs,
/// preventing it from being written to swap space. The memory buffer is
/// zeroized using cryptographic zeroization before deallocation.
pub struct LockedBuffer {
    ptr: NonNull<u8>,
    size: usize,
    layout: Layout,
}

unsafe impl Send for LockedBuffer {}
unsafe impl Sync for LockedBuffer {}

impl LockedBuffer {
    /// Creates a new page-aligned, OS-locked buffer of the specified size in bytes.
    pub fn new(size: usize) -> Self {
        if size == 0 {
            let layout = Layout::new::<u8>();
            let ptr = NonNull::dangling();
            return Self {
                ptr,
                size: 0,
                layout,
            };
        }

        let page_size = get_page_size();
        // Round up size to page boundary
        let aligned_size = (size + page_size - 1) & !(page_size - 1);
        let layout = Layout::from_size_align(aligned_size, page_size)
            .expect("Failed to create page-aligned memory layout");

        let raw_ptr = unsafe { alloc(layout) };
        if raw_ptr.is_null() {
            std::alloc::handle_alloc_error(layout);
        }

        let ptr = NonNull::new(raw_ptr).expect("Allocated pointer is null");

        // Pre-fill memory with zeroes
        unsafe {
            std::ptr::write_bytes(ptr.as_ptr(), 0, aligned_size);
        }

        // Pin the pages in physical memory
        lock_memory(ptr.as_ptr(), aligned_size);

        Self { ptr, size, layout }
    }

    /// Creates a new locked buffer initialized with the contents of the slice.
    pub fn from_slice(slice: &[u8]) -> Self {
        let buf = Self::new(slice.len());
        if slice.len() > 0 {
            unsafe {
                std::ptr::copy_nonoverlapping(slice.as_ptr(), buf.ptr.as_ptr(), slice.len());
            }
        }
        buf
    }

    /// Borrows the buffer contents as a byte slice.
    pub fn as_slice(&self) -> &[u8] {
        if self.size == 0 {
            &[]
        } else {
            unsafe { std::slice::from_raw_parts(self.ptr.as_ptr(), self.size) }
        }
    }

    /// Borrows the buffer contents as a mutable byte slice.
    pub fn as_mut_slice(&mut self) -> &mut [u8] {
        if self.size == 0 {
            &mut []
        } else {
            unsafe { std::slice::from_raw_parts_mut(self.ptr.as_ptr(), self.size) }
        }
    }

    /// Returns the length of the buffer in bytes.
    #[allow(dead_code)]
    pub fn len(&self) -> usize {
        self.size
    }

    /// Returns true if the buffer is empty.
    #[allow(dead_code)]
    pub fn is_empty(&self) -> bool {
        self.size == 0
    }
}

impl Clone for LockedBuffer {
    fn clone(&self) -> Self {
        Self::from_slice(self.as_slice())
    }
}

impl zeroize::Zeroize for LockedBuffer {
    fn zeroize(&mut self) {
        if self.size > 0 {
            let slice = unsafe { std::slice::from_raw_parts_mut(self.ptr.as_ptr(), self.layout.size()) };
            slice.zeroize();
        }
    }
}

impl Drop for LockedBuffer {
    fn drop(&mut self) {
        if self.size > 0 {
            let aligned_size = self.layout.size();

            // 1. Zeroize the memory (entire allocated page range)
            self.zeroize();

            // 2. Unlock from physical memory
            unlock_memory(self.ptr.as_ptr(), aligned_size);

            // 3. Deallocate
            unsafe {
                dealloc(self.ptr.as_ptr(), self.layout);
            }
        }
    }
}

impl std::fmt::Debug for LockedBuffer {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "LockedBuffer(<redacted>)")
    }
}

impl PartialEq for LockedBuffer {
    fn eq(&self, other: &Self) -> bool {
        if self.size != other.size {
            return false;
        }
        self.as_slice().ct_eq(other.as_slice()).unwrap_u8() == 1
    }
}

impl Eq for LockedBuffer {}

impl ConstantTimeEq for LockedBuffer {
    fn ct_eq(&self, other: &Self) -> subtle::Choice {
        if self.size != other.size {
            return subtle::Choice::from(0);
        }
        self.as_slice().ct_eq(other.as_slice())
    }
}

/// A secure UTF-8 string wrapper around `LockedBuffer` for cryptographic secrets,
/// passwords, and master keys.
#[derive(Clone, Eq)]
pub struct LockedString(LockedBuffer);

unsafe impl Send for LockedString {}
unsafe impl Sync for LockedString {}

impl LockedString {
    /// Creates a new locked string with the specified text.
    pub fn new(s: &str) -> Self {
        Self(LockedBuffer::from_slice(s.as_bytes()))
    }

    /// Returns a reference to the inner string slice.
    pub fn as_str(&self) -> &str {
        std::str::from_utf8(self.0.as_slice())
            .expect("LockedString contains invalid UTF-8")
    }
}

impl std::fmt::Debug for LockedString {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "LockedString(<redacted>)")
    }
}

impl PartialEq for LockedString {
    fn eq(&self, other: &Self) -> bool {
        self.0.eq(&other.0)
    }
}

impl ConstantTimeEq for LockedString {
    fn ct_eq(&self, other: &Self) -> subtle::Choice {
        self.0.ct_eq(&other.0)
    }
}

// ==========================================
// Platform-Specific Memory Locking Utilities
// ==========================================

#[cfg(target_os = "windows")]
fn get_page_size() -> usize {
    use windows::Win32::System::SystemInformation::{GetSystemInfo, SYSTEM_INFO};
    unsafe {
        let mut system_info = SYSTEM_INFO::default();
        GetSystemInfo(&mut system_info);
        system_info.dwPageSize as usize
    }
}

#[cfg(target_os = "windows")]
fn lock_memory(ptr: *mut u8, size: usize) {
    use std::ffi::c_void;
    use windows::Win32::System::Memory::VirtualLock;

    unsafe {
        let success = VirtualLock(ptr as *const c_void, size).is_ok();
        if !success {
            eprintln!(
                "Warning: VirtualLock failed with error: {}",
                windows::core::Error::from_win32().message()
            );
        }
    }
}

#[cfg(target_os = "windows")]
fn unlock_memory(ptr: *mut u8, size: usize) {
    use std::ffi::c_void;
    use windows::Win32::System::Memory::VirtualUnlock;

    unsafe {
        let _ = VirtualUnlock(ptr as *const c_void, size);
    }
}

#[cfg(target_os = "windows")]
pub fn lock_process_memory() {
    use windows::Win32::System::Threading::{GetCurrentProcess, SetProcessWorkingSetSize};

    unsafe {
        // Request locking between 2 MB and 20 MB working set.
        // This acts as a strong hint to Windows to keep our process pages resident in physical RAM.
        let min_size = 2 * 1024 * 1024;
        let max_size = 20 * 1024 * 1024;
        let handle = GetCurrentProcess();
        let success = SetProcessWorkingSetSize(handle, min_size, max_size).is_ok();
        if !success {
            eprintln!(
                "Warning: SetProcessWorkingSetSize failed with error: {}",
                windows::core::Error::from_win32().message()
            );
        }
    }
}

#[cfg(not(target_os = "windows"))]
fn get_page_size() -> usize {
    unsafe { libc::sysconf(libc::_SC_PAGESIZE) as usize }
}

#[cfg(not(target_os = "windows"))]
fn lock_memory(ptr: *mut u8, size: usize) {
    unsafe {
        let res = libc::mlock(ptr as *const libc::c_void, size);
        if res != 0 {
            eprintln!(
                "Warning: mlock failed with error: {}",
                std::io::Error::last_os_error()
            );
        }
    }
}

#[cfg(not(target_os = "windows"))]
fn unlock_memory(ptr: *mut u8, size: usize) {
    unsafe {
        let _ = libc::munlock(ptr as *const libc::c_void, size);
    }
}

#[cfg(not(target_os = "windows"))]
pub fn lock_process_memory() {
    unsafe {
        let res = libc::mlockall(libc::MCL_CURRENT | libc::MCL_FUTURE);
        if res != 0 {
            eprintln!(
                "Warning: mlockall failed with error: {}",
                std::io::Error::last_os_error()
            );
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_locked_buffer_lifecycle() {
        let test_data = b"secure-cryptographic-salt-or-key";
        let buf = LockedBuffer::from_slice(test_data);
        assert_eq!(buf.len(), test_data.len());
        assert_eq!(buf.as_slice(), test_data);

        // Verify clone works and holds the same data
        let cloned = buf.clone();
        assert_eq!(cloned.as_slice(), test_data);
        assert_eq!(buf, cloned);
    }

    #[test]
    fn test_locked_string_lifecycle() {
        let text = "extremely-sensitive-user-password";
        let locked_str = LockedString::new(text);
        assert_eq!(locked_str.as_str(), text);

        let cloned = locked_str.clone();
        assert_eq!(cloned.as_str(), text);
        assert_eq!(locked_str, cloned);
    }
}
