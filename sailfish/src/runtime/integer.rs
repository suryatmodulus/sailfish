#![allow(clippy::many_single_char_names)]

use std::ptr;

const DEC_DIGITS_LUT: &[u8] = b"\
      0001020304050607080910111213141516171819\
      2021222324252627282930313233343536373839\
      4041424344454647484950515253545556575859\
      6061626364656667686970717273747576777879\
      8081828384858687888990919293949596979899";

macro_rules! lookup {
    ($idx:expr) => {
        DEC_DIGITS_LUT.as_ptr().add(($idx as usize) << 1)
    };
}

/// write integer smaller than 10000
#[inline]
unsafe fn write_small(n: u16, buf: *mut u8) -> usize {
    debug_assert!(n < 10000);

    if n < 100 {
        if n < 10 {
            *buf = n as u8 + 0x30;
            1
        } else {
            ptr::copy_nonoverlapping(lookup!(n), buf, 2);
            2
        }
    } else if n < 1000 {
        let d1 = (n / 100) as u8;
        let d2 = n % 100;
        *buf = d1 + 0x30;
        ptr::copy_nonoverlapping(lookup!(d2), buf.add(1), 2);
        3
    } else {
        let d1 = n / 100;
        let d2 = n % 100;
        ptr::copy_nonoverlapping(lookup!(d1), buf, 2);
        ptr::copy_nonoverlapping(lookup!(d2), buf.add(2), 2);
        4
    }
}

/// write integer smaller with 0 padding
#[inline]
unsafe fn write_small_pad(n: u16, buf: *mut u8) {
    debug_assert!(n < 10000);

    let d1 = n / 100;
    let d2 = n % 100;

    ptr::copy_nonoverlapping(lookup!(d1), buf, 2);
    ptr::copy_nonoverlapping(lookup!(d2), buf.add(2), 2);
}

unsafe fn write_u8(n: u8, buf: *mut u8) -> usize {
    if n < 10 {
        *buf = n + 0x30;
        1
    } else if n < 100 {
        ptr::copy_nonoverlapping(lookup!(n), buf, 2);
        2
    } else {
        let d1 = n / 100;
        let d2 = n % 100;
        *buf = d1 + 0x30;
        ptr::copy_nonoverlapping(lookup!(d2), buf.add(1), 2);
        3
    }
}

unsafe fn write_u16(n: u16, buf: *mut u8) -> usize {
    if n < 100 {
        if n < 10 {
            *buf = n as u8 + 0x30;
            1
        } else {
            ptr::copy_nonoverlapping(lookup!(n), buf, 2);
            2
        }
    } else if n < 10000 {
        if n < 1000 {
            let d1 = (n / 100) as u8;
            let d2 = n % 100;
            *buf = d1 + 0x30;
            ptr::copy_nonoverlapping(lookup!(d2), buf.add(1), 2);
            3
        } else {
            let d1 = n / 100;
            let d2 = n % 100;
            ptr::copy_nonoverlapping(lookup!(d1), buf, 2);
            ptr::copy_nonoverlapping(lookup!(d2), buf.add(2), 2);
            4
        }
    } else {
        let b = (n / 10000) as u8; // 1 to 6
        let c = n % 10000;

        *buf = b + 0x30;
        write_small_pad(c, buf.add(1));
        5
    }
}

unsafe fn write_u32(mut n: u32, buf: *mut u8) -> usize {
    if n < 10000 {
        write_small(n as u16, buf)
    } else if n < 100_000_000 {
        let b = n / 10000;
        let c = n % 10000;

        let l = write_small(b as u16, buf);
        write_small_pad(c as u16, buf.add(l));
        l + 4
    } else {
        let a = n / 100_000_000; // 1 to 42
        n %= 100_000_000;

        let l = if a >= 10 {
            ptr::copy_nonoverlapping(lookup!(a), buf, 2);
            2
        } else {
            *buf = a as u8 + 0x30;
            1
        };

        let b = n / 10000;
        let c = n % 10000;

        write_small_pad(b as u16, buf.add(l));
        write_small_pad(c as u16, buf.add(l + 4));
        l + 8
    }
}

unsafe fn write_u64(mut n: u64, buf: *mut u8) -> usize {
    if n < 10000 {
        write_small(n as u16, buf)
    } else if n < 100_000_000 {
        let n = n as u32;
        let b = n / 10000;
        let c = n % 10000;

        let l = write_small(b as u16, buf);
        write_small_pad(c as u16, buf.add(l));
        l + 4
    } else if n < 10_000_000_000_000_000 {
        let v0 = n / 100_000_000;
        let v1 = (n % 100_000_000) as u32;

        let l = if v0 < 10000 {
            write_small(v0 as u16, buf)
        } else {
            let b0 = v0 / 10000;
            let c0 = v0 % 10000;
            let l = write_small(b0 as u16, buf);
            write_small_pad(c0 as u16, buf.add(l));
            l + 4
        };

        let b1 = v1 / 10000;
        let c1 = v1 % 10000;

        write_small_pad(b1 as u16, buf.add(l));
        write_small_pad(c1 as u16, buf.add(l + 4));

        l + 8
    } else {
        let a = n / 10_000_000_000_000_000; // 1 to 1844
        n %= 10_000_000_000_000_000;

        let v0 = (n / 100_000_000) as u32;
        let v1 = (n % 100_000_000) as u32;

        let b0 = v0 / 10000;
        let c0 = v0 % 10000;

        let b1 = v1 / 10000;
        let c1 = v1 % 10000;

        let l = write_small(a as u16, buf);
        write_small_pad(b0 as u16, buf.add(l));
        write_small_pad(c0 as u16, buf.add(l + 4));
        write_small_pad(b1 as u16, buf.add(l + 8));
        write_small_pad(c1 as u16, buf.add(l + 12));
        l + 16
    }
}

unsafe fn write_u128(n: u128, buf: *mut u8) -> usize {
    if n <= std::u64::MAX as u128 {
        write_u64(n as u64, buf)
    } else if n < 100_000_000_000_000_000_000_000_000_000_000 {
        let a0 = (n / 10_000_000_000_000_000) as u64;
        let a1 = (n % 10_000_000_000_000_000) as u64;

        let b0 = (a1 / 100_000_000) as u32;
        let b1 = (a1 / 100_000_000) as u32;

        let c0 = (b0 / 10000) as u16;
        let c1 = (b0 % 10000) as u16;
        let c2 = (b1 / 10000) as u16;
        let c3 = (b1 % 10000) as u16;

        let l = write_u64(a0, buf);
        write_small_pad(c0, buf.add(l));
        write_small_pad(c1, buf.add(l + 4));
        write_small_pad(c2, buf.add(l + 8));
        write_small_pad(c3, buf.add(l + 12));
        l + 16
    } else {
        let a0 = (n / 100_000_000_000_000_000_000_000_000_000_000) as u32; // 1 to 3402823
        let a1 = n % 100_000_000_000_000_000_000_000_000_000_000;

        let b0 = (a1 / 10_000_000_000_000_000) as u64;
        let b1 = (a1 % 10_000_000_000_000_000) as u64;

        let c0 = (b0 / 100_000_000) as u32;
        let c1 = (b0 % 100_000_000) as u32;
        let c2 = (b1 / 100_000_000) as u32;
        let c3 = (b1 % 100_000_000) as u32;

        let d0 = (c0 / 10000) as u16;
        let d1 = (c0 % 10000) as u16;
        let d2 = (c1 / 10000) as u16;
        let d3 = (c1 % 10000) as u16;
        let d4 = (c2 / 10000) as u16;
        let d5 = (c2 % 10000) as u16;
        let d6 = (c3 / 10000) as u16;
        let d7 = (c3 % 10000) as u16;

        let l = if a0 < 10000 {
            write_small(a0 as u16, buf)
        } else {
            let b0 = (a0 / 10000) as u16;
            let b1 = (a0 % 10000) as u16;
            let l = write_small(b0, buf);
            write_small_pad(b1, buf.add(l));
            l + 4
        };

        write_small_pad(d0, buf.add(l));
        write_small_pad(d1, buf.add(l + 4));
        write_small_pad(d2, buf.add(l + 8));
        write_small_pad(d3, buf.add(l + 12));
        write_small_pad(d4, buf.add(l + 16));
        write_small_pad(d5, buf.add(l + 20));
        write_small_pad(d6, buf.add(l + 24));
        write_small_pad(d7, buf.add(l + 28));

        l + 32
    }
}

pub trait Integer {
    const MAX_LEN: usize;
    unsafe fn write_to(self, buf: *mut u8) -> usize;
}

macro_rules! impl_integer {
    ($unsigned:ty, $signed:ty, $conv:ty, $func:ident, $max_len:expr) => {
        impl Integer for $unsigned {
            const MAX_LEN: usize = $max_len;

            #[inline]
            unsafe fn write_to(self, buf: *mut u8) -> usize {
                $func(self as $conv, buf)
            }
        }

        impl Integer for $signed {
            const MAX_LEN: usize = $max_len + 1;

            #[inline]
            unsafe fn write_to(self, mut buf: *mut u8) -> usize {
                if self >= 0 {
                    $func(self as $conv, buf)
                } else {
                    *buf = b'-';
                    buf = buf.add(1);
                    let n = (!(self as $conv)).wrapping_add(1);
                    $func(n, buf) + 1
                }
            }
        }
    };
}

impl_integer!(u8, i8, u8, write_u8, 3);
impl_integer!(u16, i16, u16, write_u16, 5);
impl_integer!(u32, i32, u32, write_u32, 10);
impl_integer!(u64, i64, u64, write_u64, 20);
impl_integer!(u128, i128, u128, write_u128, 39);

#[cfg(target_pointer_width = "16")]
impl_integer!(usize, isize, u16, write_u16, 6);

#[cfg(target_pointer_width = "32")]
impl_integer!(usize, isize, u32, write_u32, 11);

#[cfg(target_pointer_width = "64")]
impl_integer!(usize, isize, u64, write_u64, 20);

#[cfg(test)]
mod tests {
    // comprehenisive test
    #[test]
    fn test_i8_all() {
        use super::Integer;
        let mut buf = Vec::with_capacity(i8::MAX_LEN);

        for n in std::i8::MIN..=std::i8::MAX {
            unsafe {
                let l = n.write_to(buf.as_mut_ptr());
                buf.set_len(l);
                assert_eq!(std::str::from_utf8_unchecked(&*buf), format!("{}", n));
            }
        }
    }

    // random test
    #[test]
    fn test_u64_random() {
        use super::Integer;
        let mut buf = Vec::with_capacity(u64::MAX_LEN);

        let mut state = 88172645463325252u64;

        for _ in 0..100 {
            // xorshift
            state ^= state << 13;
            state ^= state >> 7;
            state ^= state << 17;

            unsafe {
                let l = state.write_to(buf.as_mut_ptr());
                buf.set_len(l);
                assert_eq!(std::str::from_utf8_unchecked(&*buf), format!("{}", state));
            }
        }
    }

    macro_rules! make_test {
        ($name:ident, $type:ty, $($value:expr),*) => {
            #[test]
            fn $name() {
                use super::Integer;

                unsafe fn test_write(val: $type, buf: &mut Vec<u8>) {
                    let l = val.write_to(buf.as_mut_ptr());
                    buf.set_len(l);
                    assert_eq!(
                        std::str::from_utf8_unchecked(&*buf),
                        format!("{}", val)
                    );
                }

                let mut buf = Vec::with_capacity(<$type>::MAX_LEN);
                unsafe {
                    $(
                        test_write($value as $type, &mut buf);
                    )*
                }
            }
        }
    }

    // boundary tests
    make_test!(test_u8, u8, 0, 1, 9, 10, 99, 100, 254, 255);
    make_test!(test_u16, u16, 0, 9, 10, 99, 100, 999, 1000, 9999, 10000, 65535);
    make_test!(
        test_u32, u32, 0, 9, 10, 99, 100, 999, 1000, 9999, 10000, 99999, 100000, 999999,
        1000000, 9999999, 10000000, 99999999, 100000000, 999999999, 1000000000,
        4294967295
    );
    make_test!(
        test_u64,
        u64,
        0,
        9,
        10,
        99,
        100,
        999,
        1000,
        9999,
        10000,
        99999,
        100000,
        999999,
        1000000,
        9999999,
        10000000,
        99999999,
        100000000,
        999999999,
        1000000000,
        9999999999,
        10000000000,
        99999999999,
        100000000000,
        999999999999,
        1000000000000,
        9999999999999,
        10000000000000,
        99999999999999,
        100000000000000,
        999999999999999,
        1000000000000000,
        9999999999999999,
        10000000000000000,
        99999999999999999,
        100000000000000000,
        999999999999999999,
        1000000000000000000,
        9999999999999999999,
        10000000000000000000,
        18446744073709551615
    );

    make_test!(
        test_u128,
        u128,
        0,
        9,
        10,
        99,
        100,
        999,
        1000,
        9999,
        10000,
        99999,
        100000,
        999999,
        1000000,
        9999999,
        10000000,
        99999999,
        100000000,
        999999999,
        1000000000,
        9999999999,
        10000000000,
        99999999999,
        100000000000,
        999999999999,
        1000000000000,
        9999999999999,
        10000000000000,
        99999999999999,
        100000000000000,
        999999999999999,
        1000000000000000,
        9999999999999999,
        10000000000000000,
        99999999999999999,
        100000000000000000,
        999999999999999999,
        1000000000000000000,
        9999999999999999999,
        10000000000000000000,
        99999999999999999999,
        100000000000000000000,
        999999999999999999999,
        1000000000000000000000,
        9999999999999999999999,
        10000000000000000000000,
        99999999999999999999999,
        100000000000000000000000,
        100000000000000000000000,
        999999999999999999999999,
        1000000000000000000000000,
        9999999999999999999999999,
        10000000000000000000000000,
        99999999999999999999999999,
        100000000000000000000000000,
        999999999999999999999999999,
        1000000000000000000000000000,
        9999999999999999999999999999,
        10000000000000000000000000000,
        99999999999999999999999999999,
        100000000000000000000000000000,
        999999999999999999999999999999,
        1000000000000000000000000000000,
        9999999999999999999999999999999,
        10000000000000000000000000000000,
        99999999999999999999999999999999,
        100000000000000000000000000000000,
        999999999999999999999999999999999,
        1000000000000000000000000000000000,
        9999999999999999999999999999999999,
        10000000000000000000000000000000000,
        99999999999999999999999999999999999,
        100000000000000000000000000000000000,
        999999999999999999999999999999999999,
        1000000000000000000000000000000000000,
        9999999999999999999999999999999999999,
        10000000000000000000000000000000000000,
        99999999999999999999999999999999999999,
        100000000000000000000000000000000000000,
        340282366920938463463374607431768211455
    );

    make_test!(test_i8, i8, std::i8::MIN, std::i8::MAX);
    make_test!(test_i16, i16, std::i16::MIN, std::i16::MAX);
    make_test!(test_i32, i32, std::i32::MIN, std::i32::MAX);
    make_test!(test_i64, i64, std::i64::MIN, std::i64::MAX);
    make_test!(test_i128, i128, std::i128::MIN, std::i128::MAX);
}
