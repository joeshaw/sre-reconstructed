// Byte layouts of the data files' records.  The original's structures
// are byte-packed and little-endian (Borland C++); the reconstruction
// keeps them as ordinary structs and converts to and from their exact
// bytes when they are checksummed, encrypted, read or written.

pub trait Le: Sized {
    const SIZE: usize;
    fn rd(b: &[u8]) -> Self;
    fn wr(&self, b: &mut [u8]);

    fn to_bytes(&self) -> Vec<u8> {
        let mut v = vec![0u8; Self::SIZE];
        self.wr(&mut v);
        v
    }
    fn from_bytes(b: &[u8]) -> Self {
        Self::rd(b)
    }
    fn zeroed() -> Self {
        Self::rd(&vec![0u8; Self::SIZE])
    }
}

macro_rules! le_int {
    ($($t:ty),*) => {$(
        impl Le for $t {
            const SIZE: usize = std::mem::size_of::<$t>();
            fn rd(b: &[u8]) -> Self {
                let mut a = [0u8; std::mem::size_of::<$t>()];
                a.copy_from_slice(&b[..Self::SIZE]);
                <$t>::from_le_bytes(a)
            }
            fn wr(&self, b: &mut [u8]) {
                b[..Self::SIZE].copy_from_slice(&self.to_le_bytes());
            }
        }
    )*};
}
le_int!(i8, u8, i16, u16, i32, u32);

impl<T: Le, const N: usize> Le for [T; N] {
    const SIZE: usize = T::SIZE * N;
    fn rd(b: &[u8]) -> Self {
        std::array::from_fn(|i| T::rd(&b[i * T::SIZE..]))
    }
    fn wr(&self, b: &mut [u8]) {
        for (i, x) in self.iter().enumerate() {
            x.wr(&mut b[i * T::SIZE..]);
        }
    }
}

// A record: the fields in order, each with its type.
macro_rules! record {
    ($(#[$m:meta])* pub struct $name:ident { $($(#[$fm:meta])* pub $f:ident : $t:ty,)* }) => {
        $(#[$m])*
        #[derive(Clone)]
        pub struct $name { $($(#[$fm])* pub $f: $t,)* }
        impl $crate::layout::Le for $name {
            const SIZE: usize = 0 $(+ <$t as $crate::layout::Le>::SIZE)*;
            #[allow(unused_assignments)]
            fn rd(b: &[u8]) -> Self {
                let mut o = 0usize;
                $(let $f = <$t as $crate::layout::Le>::rd(&b[o..]); o += <$t as $crate::layout::Le>::SIZE;)*
                $name { $($f,)* }
            }
            #[allow(unused_assignments)]
            fn wr(&self, b: &mut [u8]) {
                let mut o = 0usize;
                $(<$t as $crate::layout::Le>::wr(&self.$f, &mut b[o..]); o += <$t as $crate::layout::Le>::SIZE;)*
            }
        }
        impl Default for $name {
            fn default() -> Self { <$name as $crate::layout::Le>::zeroed() }
        }
    };
}
pub(crate) use record;
