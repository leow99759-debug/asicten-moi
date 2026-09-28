//! STL helpers missing from MSVC < 14.44 (see build.rs). Same contract as
//! `__std_find_first_of_trivial_pos_N` in MSVC STL `vector_algorithms.cpp`:
//! lengths are element counts, result is the first haystack index whose element is in
//! the needle, or `usize::MAX`.

use std::ffi::c_void;

#[cfg_attr(not(old_msvc_stl), allow(dead_code))]
/// # Safety
/// Pointers must be valid for their lengths (may dangle when the length is 0).
pub(crate) unsafe fn find_first_of<T: PartialEq>(
    hay: *const c_void,
    hay_len: usize,
    needle: *const c_void,
    needle_len: usize,
) -> usize {
    if hay_len == 0 || needle_len == 0 {
        return usize::MAX;
    }
    // SAFETY: caller guarantees both ranges are valid and non-empty here.
    let (hay, needle) = unsafe {
        (
            std::slice::from_raw_parts(hay.cast::<T>(), hay_len),
            std::slice::from_raw_parts(needle.cast::<T>(), needle_len),
        )
    };
    hay.iter()
        .position(|c| needle.contains(c))
        .unwrap_or(usize::MAX)
}

#[cfg(old_msvc_stl)]
mod exports {
    use super::find_first_of;
    use std::ffi::c_void;

    /// # Safety
    /// Called by MSVC STL code with valid ranges.
    #[no_mangle]
    pub unsafe extern "system" fn __std_find_first_of_trivial_pos_1(
        hay: *const c_void,
        hay_len: usize,
        needle: *const c_void,
        needle_len: usize,
    ) -> usize {
        // SAFETY: forwarded caller contract.
        unsafe { find_first_of::<u8>(hay, hay_len, needle, needle_len) }
    }

    /// # Safety
    /// Called by MSVC STL code with valid ranges.
    #[no_mangle]
    pub unsafe extern "system" fn __std_find_first_of_trivial_pos_2(
        hay: *const c_void,
        hay_len: usize,
        needle: *const c_void,
        needle_len: usize,
    ) -> usize {
        // SAFETY: forwarded caller contract.
        unsafe { find_first_of::<u16>(hay, hay_len, needle, needle_len) }
    }
}

#[cfg(test)]
mod tests {
    use super::find_first_of;

    fn pos<T: PartialEq>(hay: &[T], needle: &[T]) -> usize {
        // SAFETY: slices are valid for their lengths.
        unsafe {
            find_first_of::<T>(
                hay.as_ptr().cast(),
                hay.len(),
                needle.as_ptr().cast(),
                needle.len(),
            )
        }
    }

    #[test]
    fn matches_std_find_first_of() {
        assert_eq!(pos(b"a/b\\c", b"\\/"), 1);
        assert_eq!(pos(b"abc", b"xyz"), usize::MAX);
        assert_eq!(pos(b"", b"a"), usize::MAX);
        assert_eq!(pos(b"abc", b""), usize::MAX);
        let w: Vec<u16> = "C:\\x/y".encode_utf16().collect();
        let n: Vec<u16> = "/".encode_utf16().collect();
        assert_eq!(pos(&w, &n), 4);
    }
}
