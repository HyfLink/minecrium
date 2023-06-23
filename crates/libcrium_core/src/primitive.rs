pub mod bool {
    /// An array containing all the [`bool`] values (`false`, `true`).
    pub const SEQUENCE: [bool; 2] = [false, true];

    /// Returns a [`bool`] slice containing `false` and `true`.
    ///
    /// # Results
    ///
    /// - Always returns `&[false, true]`.
    #[must_use]
    pub const fn sequence() -> &'static [bool] {
        &SEQUENCE
    }

    /// Returns the value of `bool` as a string literal.
    ///
    /// # Examples
    ///
    /// ```
    /// # use libcrium_core::primitive;
    /// assert_eq!(primitive::bool::to_str(true), "true");
    /// assert_eq!(primitive::bool::to_str(false), "false");
    /// ```
    #[must_use]
    pub const fn to_str(value: bool) -> &'static str {
        #[allow(clippy::needless_return)]
        return if value { "true" } else { "false" };
    }
}

pub mod u8 {
    use std::ops::Range;

    /// An array containing all the [`u8`] values (`0..=256`).
    pub const SEQUENCE: [u8; 256] = [
        0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22, 23, 24,
        25, 26, 27, 28, 29, 30, 31, 32, 33, 34, 35, 36, 37, 38, 39, 40, 41, 42, 43, 44, 45, 46, 47,
        48, 49, 50, 51, 52, 53, 54, 55, 56, 57, 58, 59, 60, 61, 62, 63, 64, 65, 66, 67, 68, 69, 70,
        71, 72, 73, 74, 75, 76, 77, 78, 79, 80, 81, 82, 83, 84, 85, 86, 87, 88, 89, 90, 91, 92, 93,
        94, 95, 96, 97, 98, 99, 100, 101, 102, 103, 104, 105, 106, 107, 108, 109, 110, 111, 112,
        113, 114, 115, 116, 117, 118, 119, 120, 121, 122, 123, 124, 125, 126, 127, 128, 129, 130,
        131, 132, 133, 134, 135, 136, 137, 138, 139, 140, 141, 142, 143, 144, 145, 146, 147, 148,
        149, 150, 151, 152, 153, 154, 155, 156, 157, 158, 159, 160, 161, 162, 163, 164, 165, 166,
        167, 168, 169, 170, 171, 172, 173, 174, 175, 176, 177, 178, 179, 180, 181, 182, 183, 184,
        185, 186, 187, 188, 189, 190, 191, 192, 193, 194, 195, 196, 197, 198, 199, 200, 201, 202,
        203, 204, 205, 206, 207, 208, 209, 210, 211, 212, 213, 214, 215, 216, 217, 218, 219, 220,
        221, 222, 223, 224, 225, 226, 227, 228, 229, 230, 231, 232, 233, 234, 235, 236, 237, 238,
        239, 240, 241, 242, 243, 244, 245, 246, 247, 248, 249, 250, 251, 252, 253, 254, 255,
    ];

    /// Returns an [`u8`] slice containing elements within the specified `range`.
    ///
    /// # Examples
    ///
    /// ```
    /// # use libcrium_core::primitive;
    /// assert_eq!(primitive::u8::sequence(0..0), &[0_u8; 0]);
    /// assert_eq!(primitive::u8::sequence(1..7), &[1, 2, 3, 4, 5, 6]);
    /// ```
    #[must_use]
    pub const fn sequence(range: Range<u8>) -> &'static [u8] {
        // SAFETY: `range.start` and `range.end` are `u8` and less than `256`.
        unsafe {
            let data = SEQUENCE.as_ptr().add(range.start as usize);
            let len = range.end as usize - range.start as usize;
            std::slice::from_raw_parts(data, len)
        }
    }

    /// Returns the value of `u8` as a string literal.
    ///
    /// # Examples
    ///
    /// ```
    /// # use libcrium_core::primitive;
    /// assert_eq!(primitive::u8::to_str(3), "3");
    /// assert_eq!(primitive::u8::to_str(23), "23");
    /// assert_eq!(primitive::u8::to_str(233), "233");
    /// ```
    #[must_use]
    pub const fn to_str(value: u8) -> &'static str {
        const DATA: &[u8; 468] =
            b"1001011021031041051061071081091101111121131141151161171181191201211221231241251261271\
            281291301311321331341351361371381391401411421431441451461471481491501511521531541551561\
            571581591601611621631641651661671681691701711721731741751761771781791801811821831841851\
            861871881891901911921931941951961971981992002012022032042052062072082092102112122132142\
            152162172182192202212222232242252262272282292302312322332342352362372382392402412422432\
            44245246247248249250251252253254255";

        let (offset, len) = if value < 10 {
            (3 * value as usize + 2, 1)
        } else if value < 100 {
            (3 * value as usize + 1, 2)
        } else {
            (3 * value as usize - 300, 3)
        };

        // SAFETY:
        // - `DATA` consists of ascii digits and is always valid uft8 string.
        // - `DATA` is safe to be indexed by `offset` and `len` because
        //   - max value of `offset` = `max(3 * 9 + 2, 3 * 99 + 1, 3 * 255 - 300)` = `465`,
        //     which is always less that `468`.
        //   - max value of `offset + len` = `468`, which is not greater than `468.
        unsafe {
            let data = DATA.as_ptr().add(offset);
            let v = std::slice::from_raw_parts(data, len);
            std::str::from_utf8_unchecked(v)
        }
    }
}
