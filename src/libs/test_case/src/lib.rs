use errors::{Error, Result};
use num_traits::{Euclid, PrimInt, WrappingSub};
use std::io::Read;

#[derive(Debug, Clone, Copy)]
pub enum Encoding {
    UTF8,
    UTF8ASCII,
    UTF16,
}

#[derive(Debug)]
pub struct TestCase {
    // Actual data of the test case
    pub data: Vec<u8>,
    // Size of the data
    pub size: usize,
    //  Data pointer to the current position in the data
    pub data_ptr: usize,
    // Energy of the test case, used when a power schedule is used
    pub energy: usize,
    // Indices of the test cases that have been accessed/used by the fuzzer
    pub accessed: Vec<usize>,
}

impl Default for TestCase {
    fn default() -> Self {
        Self {
            data: Vec::with_capacity(4096),
            size: 4096,
            data_ptr: 0,
            energy: 0,
            accessed: Vec::new(),
        }
    }
}

impl TestCase {
    pub fn new(data: &Vec<u8>) -> Self {
        Self {
            data: data.clone(),
            size: data.len(),
            data_ptr: 0,
            energy: 0,
            accessed: Vec::new(),
        }
    }
    /// Returns the data pointer.
    ///
    /// # Returns
    ///
    /// A `usize` representing the data pointer.
    pub fn get_data_pointer(&mut self) -> usize {
        self.data_ptr
    }

    /// Returns the energy value.
    ///
    /// # Returns
    ///
    /// A `usize` representing the energy value.
    pub fn get_energy(&mut self) -> usize {
        self.energy
    }
    /// Sets the energy value.
    ///
    /// # Arguments
    ///
    /// * `energy` - A `usize` representing the energy value.
    ///
    /// # Returns
    ///
    /// The modified object with the updated energy value.
    pub fn set_energy(mut self, energy: usize) -> Self {
        self.energy = energy;
        self
    }

    /// Returns the size.
    ///
    /// # Returns
    ///
    /// A `usize` representing the size.
    pub fn get_size(&mut self) -> usize {
        self.size
    }

    /// Sets the accessed indices.
    ///
    /// # Arguments
    ///
    /// * `indices` - A `Vec<usize>` containing the accessed indices.
    ///
    /// # Returns
    ///
    /// The modified object with the updated accessed indices.
    pub fn set_accessed(mut self, indices: Vec<usize>) -> Self {
        self.accessed.extend_from_slice(&indices);
        self
    }

    /// Clears the accessed indices.
    pub fn clear_accessed(&mut self) {
        self.accessed.clear();
    }
    ///
    /// Determines if the primitive integer type is signed.
    ///
    /// # Type Parameters
    ///
    /// * `T: PrimInt` - A primitive integer type implementing the `PrimInt` trait.
    ///
    /// # Returns
    ///
    /// `true` if the type is signed, `false` otherwise.
    fn is_signed<T: PrimInt>() -> bool {
        T::min_value() < T::zero() && T::max_value() > T::zero()
    }

    /// Returns the maximum length considering the given length and the remaining data size.
    ///
    /// # Arguments
    ///
    /// * `len` - A `usize` representing the requested length.
    ///
    /// # Returns
    ///
    /// A `Result<usize>` containing the maximum length or an error if the requested size is not valid.
    fn _get_max(&mut self, len: usize) -> Result<usize> {
        self.is_size_sane(len)?;
        Ok(len.min(self.size - self.data_ptr))
    }

    /// Checks if the requested size is sane.
    ///
    /// # Arguments
    ///
    /// * `requested` - A `usize` representing the requested size.
    ///
    /// # Returns
    ///
    /// A `Result<()>` containing an error if the requested size is not sane.
    fn is_size_sane(&mut self, requested: usize) -> Result<()> {
        if requested + self.data_ptr > self.size {
            return Err(Error::new("Not enough data left to fullfil request"));
        }
        Ok(())
    }

    /// Consumes a single `bool` from the stream.
    ///
    /// # Returns
    ///
    /// A `Result<bool>` which is `Ok(bool)` if the operation is successful, or an `Err(Error)` if not.
    ///
    /// # Examples
    ///
    /// ```
    /// use test_case::TestCase;
    ///
    /// let mut tc = TestCase::new(&vec![0x01]);
    /// assert_eq!(tc.consume_bool().unwrap(), true);
    /// assert_eq!(tc.data_ptr, 1);
    /// ```
    pub fn consume_bool(&mut self) -> Result<bool> {
        let _max = self._get_max(1)?;
        let byte = self.consume_byte();
        if let Ok(b) = byte {
            return Ok(b & 1 == 1);
        }
        Err(Error::new("Failed to consume bool from stream"))
    }

    /// Consumes `num` `bool`s from the stream.
    ///
    /// # Arguments
    ///
    /// * `num` - A `usize` indicating the number of `bool`s to consume.
    ///
    /// # Returns
    ///
    /// A `Result<Vec<bool>>` which is `Ok(Vec<bool>)` if the operation is successful, or an `Err(Error)` if not.
    ///
    /// # Examples
    ///
    /// ```
    /// use test_case::TestCase;
    ///
    /// let mut tc = TestCase::new(&vec![0x1, 0x0]);
    /// assert_eq!(tc.consume_booleans(2).unwrap(), vec![true, false]);
    /// assert_eq!(tc.data_ptr, 2);
    /// ```
    pub fn consume_booleans(&mut self, num: usize) -> Result<Vec<bool>> {
        let max = self._get_max(num)?;
        let mut bools = vec![false; max];
        for b in &mut bools {
            if let Ok(boolean) = self.consume_bool() {
                *b = boolean;
            }
        }
        Ok(bools)
    }

    /// Consumes a single `u8` from the stream.
    ///
    /// # Returns
    ///
    /// A `Result<u8>` which is `Ok(u8)` if the operation is successful, or an `Err(Error)` if not.
    ///
    /// # Examples
    ///
    /// ```
    /// use test_case::TestCase;
    ///
    /// let mut tc = TestCase::new(&vec![0x42]);
    /// assert_eq!(tc.consume_byte().unwrap(), 0x42);
    /// assert_eq!(tc.data_ptr, 1);
    /// ```
    pub fn consume_byte(&mut self) -> Result<u8> {
        let _max = self._get_max(1)?;
        let ret = self.data[self.data_ptr];
        self.data_ptr += 1;
        Ok(ret)
    }

    /// Consumes `num` `u8`s from the stream.
    ///
    /// # Arguments
    ///
    /// * `num` - A `usize` indicating the number of `u8`s to consume.
    ///
    /// # Returns
    ///
    /// A `Result<Vec<u8>>` which is `Ok(Vec<u8>)` if the operation is successful, or an `Err(Error)` if not.
    ///
    /// # Examples
    ///
    /// ```
    /// use test_case::TestCase;
    ///
    /// let mut tc = TestCase::new(&vec![0x42, 0x24]);
    /// assert_eq!(tc.consume_bytes(2).unwrap(), vec![0x42, 0x24]);
    /// assert_eq!(tc.data_ptr, 2);
    /// ```
    pub fn consume_bytes(&mut self, num: usize) -> Result<Vec<u8>> {
        let max = self._get_max(num)?;
        let mut bytes = vec![0u8; max];
        for b in &mut bytes {
            if let Ok(byte) = self.consume_byte() {
                *b = byte;
            }
        }
        Ok(bytes)
    }

    /// Consumes the remaining bytes in the stream as a `Vec<u8>`.
    ///
    /// # Returns
    ///
    /// A `Result<Vec<u8>>` which is `Ok(Vec<u8>)` if the operation is successful, or an `Err(Error)` if not.
    ///
    /// # Examples
    ///
    /// ```
    /// use test_case::TestCase;
    ///
    /// let mut tc = TestCase::new(&vec![0x42, 0x24]);
    /// assert_eq!(tc.consume_remaining_as_bytes().unwrap(), vec![0x42, 0x24]);
    /// assert_eq!(tc.data_ptr, 2);
    /// ```
    pub fn consume_remaining_as_bytes(&mut self) -> Result<Vec<u8>> {
        self.consume_bytes(self.size - self.data_ptr)
    }

    /// Consumes a `String` of the specified length and encoding from the stream.
    ///
    /// # Arguments
    ///
    /// * `len` - A `usize` indicating the length of the `String` to consume.
    /// * `encoding` - The `Encoding` variant to interpret the consumed bytes.
    ///
    /// # Returns
    ///
    /// A `Result<String>` which is `Ok(String)` if the operation is successful, or an `Err(Error)` if not.
    ///
    /// # Examples
    ///
    /// ```
    /// use test_case::TestCase;
    /// use test_case::Encoding;
    ///
    /// let mut tc = TestCase::new(&vec![0x48, 0x65, 0x6c, 0x6c, 0x6f]);
    /// assert_eq!(tc.consume_str(5, Encoding::UTF8).unwrap(), "Hello");
    /// assert_eq!(tc.data_ptr, 5);
    /// ```
    pub fn consume_str(&mut self, len: usize, encoding: Encoding) -> Result<String> {
        let end = match encoding {
            Encoding::UTF8 | Encoding::UTF8ASCII => self._get_max(len)?,
            Encoding::UTF16 => self._get_max(len * 2)?,
        };
        let slice = &mut self.data[self.data_ptr..self.data_ptr + end];
        let s = match encoding {
            Encoding::UTF8 => String::from_utf8_lossy(slice).to_string(),
            Encoding::UTF8ASCII => {
                let slice = slice
                    .iter_mut()
                    .map(|byte| byte.wrapping_sub(32) % 95 + 32)
                    .collect::<Vec<_>>();
                String::from_utf8_lossy(&slice).to_string()
            }
            Encoding::UTF16 => {
                let utf16_slice =
                    unsafe { std::slice::from_raw_parts(slice.as_ptr().cast::<u16>(), end) };
                String::from_utf16_lossy(utf16_slice)
            }
        };

        self.data_ptr += end;

        Ok(s)
    }

    /// Consumes the remaining data in the stream as a string with the specified encoding.
    ///
    /// # Arguments
    ///
    /// * `encoding` - The `Encoding` used for the string.
    ///
    /// # Returns
    ///
    /// A `Result<String>` which is `Ok(String)` if the operation is successful, or an `Err(Error)` if not.
    ///
    /// # Examples
    ///
    /// ```
    /// use test_case::TestCase;
    /// use test_case::Encoding;
    ///
    /// let mut tc = TestCase::new(&vec![0x48, 0x65, 0x6c, 0x6c, 0x6f, 0x42, 0x24]);
    /// assert_eq!(tc.consume_str(5, Encoding::UTF8).unwrap(), "Hello");
    /// assert_eq!(tc.consume_remaining_as_str(Encoding::UTF8).unwrap(), "B$");
    /// assert_eq!(tc.data_ptr, 7);
    /// ```
    pub fn consume_remaining_as_str(&mut self, encoding: Encoding) -> Result<String> {
        self.consume_str(self.size - self.data_ptr, encoding)
    }

    /// Consumes a single integer of type `T` from the stream with the specified endianness.
    ///
    /// # Arguments
    ///
    /// * `is_little_endian` - A `bool` indicating the endianness of the integer.
    ///
    /// # Returns
    ///
    /// A `Result<T>` which is `Ok(T)` if the operation is successful, or an `Err(Error)` if not.
    ///
    /// # Examples
    ///
    /// ```
    /// use test_case::TestCase;
    ///
    /// let mut tc = TestCase::new(&vec![0x00, 0x01, 0x02, 0x03]);
    /// assert_eq!(tc.consume_int::<u16>(true).unwrap(), 0x0100);
    /// assert_eq!(tc.data_ptr, 2);
    ///
    /// ```
    pub fn consume_int<T: PrimInt>(&mut self, is_little_endian: bool) -> Result<T> {
        let is_signed =
            std::num::Wrapping(T::min_value()) < std::num::Wrapping(T::from(0).unwrap());
        if is_signed {
            self._consume_int_s(is_little_endian)
        } else {
            self._consume_int_u(is_little_endian)
        }
    }

    /// Consumes `num` integers of type `T` from the stream with the specified endianness.
    ///
    /// # Arguments
    ///
    /// * `is_little_endian` - A `bool` indicating the endianness of the integers.
    /// * `num` - A `usize` indicating the number of integers to consume.
    ///
    /// # Returns
    ///
    /// A `Result<Vec<T>>` which is `Ok(Vec<T>)` if the operation is successful, or an `Err(Error)` if not.
    ///
    /// # Examples
    ///
    /// ```
    /// use test_case::TestCase;
    ///
    /// let mut tc = TestCase::new(&vec![0x00, 0x01, 0x00, 0x02, 0x00, 0x03, 0x00, 0x04]);
    /// assert_eq!(tc.consume_ints::<u16>(true, 2).unwrap(), vec![0x0100, 0x0200]);
    /// assert_eq!(tc.data_ptr, 4);
    /// assert_eq!(tc.consume_ints::<u16>(false, 2).unwrap(), vec![0x0003, 0x0004]);
    /// assert_eq!(tc.data_ptr, 8);
    /// ```
    pub fn consume_ints<T: PrimInt>(
        &mut self,
        is_little_endian: bool,
        num: usize,
    ) -> Result<Vec<T>> {
        let max = std::cmp::min(num, self._get_max(std::mem::size_of::<T>() * num)?);
        let mut nums: Vec<T> = vec![T::from(0).unwrap(); max];
        (0..nums.len()).for_each(|n| {
            if let Ok(num) = self.consume_int(is_little_endian) {
                nums[n] = num;
            }
        });
        Ok(nums)
    }

    /// Consumes `num` integers of type `T` from the stream with the specified endianness, within the given range.
    ///
    /// # Arguments
    ///
    /// * `is_little_endian` - A `bool` indicating the endianness of the integers.
    /// * `num` - A `usize` indicating the number of integers to consume.
    /// * `min` - A `T` indicating the minimum value of the range.
    /// * `max` - A `T` indicating the maximum value of the range.
    ///
    /// # Returns
    ///
    /// A `Result<Vec<T>>` which is `Ok(Vec<T>)` if the operation is successful, or an `Err(Error)` if not.
    ///
    /// # Examples
    ///
    /// ```
    /// use test_case::TestCase;
    ///
    /// let mut tc = TestCase::new(&vec![0x00, 0x01, 0x00, 0x02]);
    /// let result = tc.consume_ints_range::<u16>(true, 2, 0x0100, 0x0200).unwrap();
    /// assert!(result[0] >= 0x0100 && result[0] <= 0x0200);
    /// assert!(result[1] >= 0x0100 && result[1] <= 0x0200);
    /// assert_eq!(tc.data_ptr, 4);
    /// ```
    pub fn consume_ints_range<T: PrimInt + Euclid + WrappingSub>(
        &mut self,
        is_little_endian: bool,
        num: usize,
        min: T,
        max: T,
    ) -> Result<Vec<T>> {
        let max_ele = std::cmp::min(num, self._get_max(std::mem::size_of::<T>() * num)?);
        let mut nums: Vec<T> = vec![T::from(0).unwrap(); max_ele];
        (0..nums.len()).for_each(|n| {
            if let Ok(num) = self.consume_int_range(is_little_endian, min, max) {
                nums[n] = num;
            }
        });
        Ok(nums)
    }
    /// Consumes a single integer of type `T` from the stream with the specified endianness, within the given range.
    ///
    /// # Arguments
    ///
    /// * `is_little_endian` - A `bool` indicating the endianness of the integer.
    /// * `min` - A `T` indicating the minimum value of the range.
    /// * `max` - A `T` indicating the maximum value of the range.
    ///
    /// # Returns
    ///
    /// A `Result<T>` which is `Ok(T)` if the operation is successful, or an `Err(Error)` if not.
    ///
    /// # Examples
    ///
    /// ```
    /// use test_case::TestCase;
    ///
    /// let mut tc = TestCase::new(&vec![0x00, 0x01]);
    /// let result = tc.consume_int_range::<u16>(true, 0x0100, 0x0200).unwrap();
    /// assert!(result >= 0x0100 && result <= 0x0200);
    /// assert_eq!(tc.data_ptr, 2);
    /// ```
    pub fn consume_int_range<T: PrimInt + Euclid + WrappingSub>(
        &mut self,
        is_little_endian: bool,
        min: T,
        max: T,
    ) -> Result<T> {
        if max == min {
            return Ok(T::from(min).unwrap());
        }
        assert!(min < max, "min must be less than max");

        let range = max.wrapping_sub(&min);

        if Self::is_signed::<T>() {
            let signed_min = self._consume_int_s::<T>(is_little_endian)?;
            let wrapped = if range == T::max_value()
                || range == T::max_value().wrapping_sub(&T::min_value())
            {
                signed_min
            } else {
                signed_min.rem_euclid(&(range + T::from(1).unwrap()))
            };
            Ok(min + wrapped)
        } else {
            let unsigned_min = self._consume_int_u::<T>(is_little_endian)?;
            let wrapped = if range == T::max_value() {
                unsigned_min
            } else {
                unsigned_min.rem_euclid(&(range + T::from(1).unwrap()))
            };
            Ok(min + wrapped)
        }
    }

    /// Consumes a single integer of type `T` from the stream as an unsigned integer with the specified endianness.
    fn _consume_int_u<T: PrimInt>(&mut self, is_little_endian: bool) -> Result<T> {
        let bytes = std::mem::size_of::<T>();
        let vals = self.consume_bytes(bytes)?;
        match bytes {
            1 => Ok(T::from(vals[0]).unwrap()),
            2 => {
                let ret = if is_little_endian {
                    u16::from_le_bytes(vals.try_into().unwrap())
                } else {
                    u16::from_be_bytes(vals.try_into().unwrap())
                };
                Ok(T::from(ret).unwrap())
            }
            4 => {
                let ret = if is_little_endian {
                    u32::from_le_bytes(vals.try_into().unwrap())
                } else {
                    u32::from_be_bytes(vals.try_into().unwrap())
                };
                Ok(T::from(ret).unwrap())
            }
            8 => {
                let ret = if is_little_endian {
                    u64::from_le_bytes(vals.try_into().unwrap())
                } else {
                    u64::from_be_bytes(vals.try_into().unwrap())
                };
                Ok(T::from(ret).unwrap())
            }
            16 => {
                let ret = if is_little_endian {
                    u128::from_le_bytes(vals.try_into().unwrap())
                } else {
                    u128::from_be_bytes(vals.try_into().unwrap())
                };
                Ok(T::from(ret).unwrap())
            }
            _ => {
                unreachable!()
            }
        }
    }

    /// Consumes a single integer of type `T` from the stream as a signed integer with the specified endianness.
    fn _consume_int_s<T: PrimInt>(&mut self, is_little_endian: bool) -> Result<T> {
        let bytes = std::mem::size_of::<T>();
        let max_val = (1u128 << (bytes * 8 - 1)) - 1;
        match bytes {
            1 => {
                let ret = self._consume_int_u::<u8>(is_little_endian)?;
                Ok(T::from(ret % max_val as u8).unwrap())
            }
            2 => {
                let ret = self._consume_int_u::<u16>(is_little_endian)?;
                Ok(T::from(ret % max_val as u16).unwrap())
            }
            4 => {
                let ret = self._consume_int_u::<u32>(is_little_endian)?;
                Ok(T::from(ret % max_val as u32).unwrap())
            }
            8 => {
                let ret = self._consume_int_u::<u64>(is_little_endian)?;
                Ok(T::from(ret % max_val as u64).unwrap())
            }
            16 => {
                let ret = self._consume_int_u::<u128>(is_little_endian)?;
                Ok(T::from(ret % max_val).unwrap())
            }
            _ => unreachable!(),
        }
    }

    /// Consumes an IEEE 754 floating-point number from the input data.
    /// The number is read as is, without any conversion.
    ///
    /// # Returns
    ///
    /// A `f64` representing the consumed number. The consumed number may have a special value (e.g. NaN or infinity).
    ///
    /// # Example
    ///
    /// ```
    /// use test_case::TestCase;
    /// let data = [1,2,3,4,5,6,7,8].to_vec();
    /// let mut tc = TestCase::new(&data);
    /// let num = tc.consume_float();
    /// assert!(num.is_ok());
    /// assert_eq!(num.unwrap(), 5.447603722011605e-270);
    /// assert_eq!(tc.data_ptr, 8);
    /// ```
    pub fn consume_float(&mut self) -> Result<f64> {
        if self.data_ptr == self.size {
            return Ok(0.0);
        }
        if self.data_ptr + 8 > self.size {
            let mut cdata = [0u8; 8];
            let data_slice = &self.data[self.data_ptr..self.data_ptr + (self.size - self.data_ptr)];
            let mut reader = std::io::Cursor::new(data_slice);
            let bytes_read = reader.read(&mut cdata[..]).unwrap();
            cdata[bytes_read..].iter_mut().for_each(|c| *c = 0);
            cdata.reverse();
            self.data_ptr = self.size;
            Ok(f64::from_bits(u64::from_le_bytes(cdata)))
        } else {
            let ret = f64::from_bits(u64::from_le_bytes(
                self.data[self.data_ptr..self.data_ptr + 8]
                    .try_into()
                    .unwrap(),
            ));

            self.data_ptr += 8;
            Ok(ret)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::mem::size_of;

    #[test]
    fn test_consume_ints_range_limits() {
        let mut tc = setup();
        let x = tc.consume_int_range::<u8>(false, u8::MIN, u8::MAX);
        assert!(x.is_ok());
        let x = tc.consume_int_range::<u16>(false, u16::MIN, u16::MAX);
        assert!(x.is_ok());
        let x = tc.consume_int_range::<u32>(false, u32::MIN, u32::MAX);
        assert!(x.is_ok());
        let x = tc.consume_int_range::<u64>(false, u64::MIN, u64::MAX);
        assert!(x.is_ok());
        let x = tc.consume_int_range::<u128>(false, u128::MIN, u128::MAX);
        assert!(x.is_ok());
        assert_eq!(
            tc.data_ptr,
            size_of::<u128>()
                + size_of::<u64>()
                + size_of::<u32>()
                + size_of::<u16>()
                + size_of::<u8>()
        );

        let cur_ptr = tc.data_ptr;
        let x = tc.consume_int_range::<i8>(false, i8::MIN, i8::MAX);
        assert!(x.is_ok());
        let x = tc.consume_int_range::<i16>(false, i16::MIN, i16::MAX);
        assert!(x.is_ok());
        let x = tc.consume_int_range::<i32>(false, i32::MIN, i32::MAX);
        assert!(x.is_ok());
        let x = tc.consume_int_range::<i64>(false, i64::MIN, i64::MAX);
        assert!(x.is_ok());
        let x = tc.consume_int_range::<i128>(false, i128::MIN, i128::MAX);
        assert!(x.is_ok());
        assert_eq!(
            tc.data_ptr,
            cur_ptr
                + size_of::<i128>()
                + size_of::<i64>()
                + size_of::<i32>()
                + size_of::<i16>()
                + size_of::<i8>()
        );
    }

    #[test]
    fn test_remaining_bytes() {
        let mut tc = setup();
        assert_eq!(tc.size - tc.data_ptr, 1024);
        let _ = tc.consume_bool();
        assert_eq!(tc.size - tc.data_ptr, 1023);
        let _ = tc.consume_bool();
        let _ = tc.consume_bool();
        assert_eq!(tc.size - tc.data_ptr, 1021);
        let _ = tc.consume_bool();
        assert_eq!(tc.size - tc.data_ptr, 1020);
        let _ = tc.consume_byte();
        assert_eq!(tc.size - tc.data_ptr, 1019);
    }

    #[test]
    fn test_consume_booleans() {
        let mut tc = setup();
        let b = tc.consume_booleans(5);
        assert_eq!([false, true, true, false, true], b.unwrap().as_slice());
        assert_eq!(tc.size - tc.data_ptr, 1019);
        let _ = tc.consume_float();
        assert_eq!(tc.size - tc.data_ptr, 1011);
        let b = tc.consume_booleans(5);
        assert_eq!([false, false, true, false, false], b.unwrap().as_slice());
        assert_eq!(tc.size - tc.data_ptr, 1006);
        let _b = tc.consume_booleans(1000);
        assert_eq!(tc.size - tc.data_ptr, 6);
    }

    #[test]
    fn test_consume_byte() {
        let mut tc = setup();
        for i in 0..128 {
            assert_eq!(tc.consume_byte().unwrap(), tc.data[i]);
            assert_eq!(tc.size - tc.data_ptr, 1024 - 1 - i);
        }
    }

    #[test]
    fn test_consume_bytes() {
        let mut tc = setup();
        let step = 16;
        let mut ctr = 1;
        for i in (0..128).step_by(step) {
            let ret = tc.consume_bytes(step);
            assert!(ret.is_ok());
            assert_eq!(ret.unwrap(), tc.data[i..i + 16]);
            assert_eq!(tc.size - tc.data_ptr, 1024 - (step * ctr));
            ctr += 1;
        }
    }

    #[test]
    fn test_consume_rem_bytes() {
        let mut tc = setup();
        let b = tc.consume_remaining_as_bytes();
        assert!(b.is_ok());
        assert_eq!(b.unwrap(), tc.data[..]);
        assert_eq!(tc.data_ptr, tc.size);
        let b = tc.consume_bytes(16);
        assert!(b.is_err());
        match b {
            Err(Error::ConsumeError(s)) => assert_eq!(s, "Not enough data left to fullfil request"),
            _ => panic!("Unexpected error type!"),
        }
    }

    #[test]
    fn test_consume_int_u_le() {
        let mut tc = setup();
        let b = tc.consume_int::<u8>(true);
        assert!(b.is_ok());
        assert_eq!(b.unwrap(), 0x8a);
        assert_eq!(tc.data_ptr, 1);
        let b = tc.consume_int::<u16>(true);
        assert!(b.is_ok());
        assert_eq!(b.unwrap(), 0x0d19);
        assert_eq!(tc.data_ptr, 3);
        let b = tc.consume_int::<u32>(true);
        assert!(b.is_ok());
        assert_eq!(b.unwrap(), 0x380d3744);
        assert_eq!(tc.data_ptr, 7);
        let b = tc.consume_int::<u64>(true);
        assert!(b.is_ok());
        assert_eq!(b.unwrap(), 0xf288aadaf3aa9b5e);
        assert_eq!(tc.data_ptr, 15);
        let b = tc.consume_int::<u128>(true);
        assert!(b.is_ok());
        assert_eq!(b.unwrap(), 0xd090c91c7f1aacb813cff2b1beba6c9b);
        assert_eq!(tc.data_ptr, 31);
    }

    #[test]
    fn test_consume_int_u_be() {
        let mut tc = setup();
        let b = tc.consume_int::<u8>(false);
        assert!(b.is_ok());
        assert_eq!(b.unwrap(), 0x8a);
        assert_eq!(tc.data_ptr, 1);
        let b = tc.consume_int::<u16>(false);
        assert!(b.is_ok());
        assert_eq!(b.unwrap(), 0x190d);
        assert_eq!(tc.data_ptr, 3);
        let b = tc.consume_int::<u32>(false);
        assert!(b.is_ok());
        assert_eq!(b.unwrap(), 0x44370d38);
        assert_eq!(tc.data_ptr, 7);
        let b = tc.consume_int::<u64>(false);
        assert!(b.is_ok());
        assert_eq!(b.unwrap(), 0x5e9baaf3daaa88f2);
        assert_eq!(tc.data_ptr, 15);
        let b = tc.consume_int::<u128>(false);
        assert!(b.is_ok());
        assert_eq!(b.unwrap(), 0x9b6cbabeb1f2cf13b8ac1a7f1cc990d0);
        assert_eq!(tc.data_ptr, 31);
    }

    #[test]
    fn test_consume_int_s_le() {
        let mut tc = setup();
        let b = tc.consume_int::<i8>(true);
        assert!(b.is_ok());
        assert_eq!(b.unwrap(), 0xb);
        assert_eq!(tc.data_ptr, 1);
        let b = tc.consume_int::<i16>(true);
        assert!(b.is_ok());
        assert_eq!(b.unwrap(), 0xd19);
        assert_eq!(tc.data_ptr, 3);
        let b = tc.consume_int::<i32>(true);
        assert!(b.is_ok());
        assert_eq!(b.unwrap(), 0x380d3744);
        assert_eq!(tc.data_ptr, 7);
        let b = tc.consume_int::<i64>(true);
        assert!(b.is_ok());
        assert_eq!(b.unwrap(), 0x7288aadaf3aa9b5f);
        assert_eq!(tc.data_ptr, 15);
        let b = tc.consume_int::<i128>(true);
        assert!(b.is_ok());
        assert_eq!(b.unwrap(), 0x5090c91c7f1aacb813cff2b1beba6c9c);
        assert_eq!(tc.data_ptr, 31);
    }

    #[test]
    fn test_consume_int_s_be() {
        let mut tc = setup();
        let b = tc.consume_int::<i8>(false);
        assert!(b.is_ok());
        assert_eq!(b.unwrap(), 0xb);
        assert_eq!(tc.data_ptr, 1);
        let b = tc.consume_int::<i16>(false);
        assert!(b.is_ok());
        assert_eq!(b.unwrap(), 0x190d);
        assert_eq!(tc.data_ptr, 3);
        let b = tc.consume_int::<i32>(false);
        assert!(b.is_ok());
        assert_eq!(b.unwrap(), 0x44370d38);
        assert_eq!(tc.data_ptr, 7);
        let b = tc.consume_int::<i64>(false);
        assert!(b.is_ok());
        assert_eq!(b.unwrap(), 0x5e9baaf3daaa88f2);
        assert_eq!(tc.data_ptr, 15);
        let b = tc.consume_int::<i128>(false);
        assert!(b.is_ok());
        assert_eq!(b.unwrap(), 0x1b6cbabeb1f2cf13b8ac1a7f1cc990d1);
        assert_eq!(tc.data_ptr, 31);
    }

    #[test]
    fn test_consume_int_range() {
        let mut tc = setup();
        let b = tc.consume_int_range::<i8>(false, 80, 120);
        assert!(b.is_ok());
        assert_eq!(b.unwrap(), 91);
        assert_eq!(tc.data_ptr, 1);
        let b = tc.consume_int_range::<i16>(false, 80, 256);
        assert!(b.is_ok());
        assert_eq!(b.unwrap(), 121);
        assert_eq!(tc.data_ptr, 3);
        let b = tc.consume_int_range::<i32>(false, 10000, 20000);
        assert!(b.is_ok());
        assert_eq!(b.unwrap(), 14118);
        assert_eq!(tc.data_ptr, 7);
        let b = tc.consume_int_range::<i64>(false, -1000, 1000);
        assert!(b.is_ok());
        assert_eq!(b.unwrap(), 278);
        assert_eq!(tc.data_ptr, 15);
        let b = tc.consume_int_range::<i128>(false, 10, 15);
        assert!(b.is_ok());
        assert_eq!(b.unwrap(), 13);
        assert_eq!(tc.data_ptr, 31);
        let b = tc.consume_int_range::<u8>(false, 253, 255);
        assert!(b.is_ok());
        assert_eq!(b.unwrap(), 254);
        assert_eq!(tc.data_ptr, 32);
        let b = tc.consume_int_range::<u16>(false, 80, 256);
        assert!(b.is_ok());
        assert_eq!(b.unwrap(), 157);
        assert_eq!(tc.data_ptr, 34);
        let b = tc.consume_int_range::<u32>(false, 10000, 20000);
        assert!(b.is_ok());
        assert_eq!(b.unwrap(), 18444);
        assert_eq!(tc.data_ptr, 38);
        let b = tc.consume_int_range::<u64>(false, 0, 1000);
        assert!(b.is_ok());
        assert_eq!(b.unwrap(), 910);
        assert_eq!(tc.data_ptr, 46);
        let b = tc.consume_int_range::<u128>(false, 10, 15);
        assert!(b.is_ok());
        assert_eq!(b.unwrap(), 13);
        assert_eq!(tc.data_ptr, 62);
    }

    #[test]
    fn test_consume_ints_range() {
        let mut tc = setup();
        let b = tc.consume_ints_range::<i8>(false, 12, 80, 120);
        assert!(b.is_ok());
        assert_eq!(
            b.unwrap(),
            [91, 105, 93, 107, 94, 93, 95, 92, 108, 82, 114, 89]
        );
        assert_eq!(tc.data_ptr, 12);
        let b = tc.consume_ints_range::<u32>(false, 8, u32::MIN, 1025);
        assert!(b.is_ok());
        assert_eq!(b.unwrap(), [175, 115, 196, 206, 701, 425, 471, 367]);
        assert_eq!(tc.data_ptr, 44);
    }

    #[test]
    fn test_consume_str() {
        let mut tc = setup();
        let s = tc.consume_str(16, Encoding::UTF8);
        assert!(s.is_ok());
        assert_eq!(s.unwrap(), "�\u{19}\rD7\r8^���ڪ��".to_string());
        assert_eq!(tc.data_ptr, 16);

        let s = tc.consume_str(64, Encoding::UTF8);
        assert!(s.is_ok());
        assert_eq!(
            s.unwrap(),
            "l�����\u{13}��\u{1a}\u{7f}\u{1c}ɐ��\\B���\u{5}�\u{3}7IPK�9�\tl/�ѵG���y��nQ���@J�%z'Ȓ�0�@ff"
                .to_string()
        );
        assert_eq!(tc.data_ptr, 80);
    }

    #[test]
    fn test_consume_rem_str() {
        let mut tc = setup();
        let _ = tc.consume_bytes(900);
        assert_eq!(tc.data_ptr, 1024 - (1024 - 900));

        let s = tc.consume_remaining_as_str(Encoding::UTF8);
        assert!(s.is_ok());
        assert_eq!(
            s.unwrap(),
            "��\u{1e}��;\\ݔ\u{3}�\u{18},��7�S(`�w�;����S-�\u{19}~��F���Mm[�Vk\u{12}Ucë\u{8}�.�\u{11}�\u{18}ˋ\u{12}.>u2���<�3F�z�\u{12}\t&~~\u{3}O����O��9b��3�-�0o��a�2�0�Q�\u{1f}:\u{11}M�T�=Cs9\u{16}�=)J".to_string());
        assert_eq!(tc.data_ptr, 1024);
    }

    #[test]
    fn test_consume_float() {
        let mut tc = setup();
        tc.data = [0, 0, 0, 0, 0, 0, 0xf0, 0x3f, 0xa].to_vec();
        let b = tc.consume_float();
        assert!(b.is_ok());
        assert_eq!(b.unwrap(), 1.0);
        assert_eq!(tc.data_ptr, 8);
        assert_eq!(tc.consume_byte().unwrap(), 0xa);

        reset_with_data(&mut tc, [1, 0, 0, 0, 0, 0, 0xf0, 0x3f].to_vec());
        let b = tc.consume_float();
        assert!(b.is_ok());
        assert_eq!(b.unwrap(), 1.0000000000000002);

        reset_with_data(&mut tc, [0, 0, 0, 0, 0, 0, 0, 0x40].to_vec());
        let b = tc.consume_float();
        assert!(b.is_ok());
        assert_eq!(b.unwrap(), 2.0);

        reset_with_data(&mut tc, [0, 0, 0, 0, 0, 0, 0, 0xc0].to_vec());
        let b = tc.consume_float();
        assert!(b.is_ok());
        assert_eq!(b.unwrap(), -2.0);

        reset_with_data(&mut tc, [0, 0, 0, 0, 0, 0, 8, 0x40].to_vec());
        let b = tc.consume_float();
        assert!(b.is_ok());
        assert_eq!(b.unwrap(), 3.0);

        reset_with_data(&mut tc, [0, 0, 0, 0, 0, 0, 0x10, 0x40].to_vec());
        let b = tc.consume_float();
        assert!(b.is_ok());
        assert_eq!(b.unwrap(), 4.0);

        reset_with_data(&mut tc, [0, 0, 0, 0, 0, 0, 0x14, 0x40].to_vec());
        let b = tc.consume_float();
        assert!(b.is_ok());
        assert_eq!(b.unwrap(), 5.0);

        reset_with_data(&mut tc, [0, 0, 0, 0, 0, 0, 0, 0].to_vec());
        let b = tc.consume_float();
        assert!(b.is_ok());
        assert_eq!(b.unwrap(), 0.0);

        reset_with_data(&mut tc, [0, 0, 0, 0, 0, 0, 0xf0, 0x7f].to_vec());
        let b = tc.consume_float();
        assert!(b.is_ok());
        assert_eq!(b.unwrap(), f64::INFINITY);

        reset_with_data(&mut tc, [0, 0, 0, 0, 0, 0, 0xf0, 0xff].to_vec());
        let b = tc.consume_float();
        assert!(b.is_ok());
        assert_eq!(b.unwrap(), f64::NEG_INFINITY);

        reset_with_data(&mut tc, [1, 0, 0, 0, 0, 0, 0xf0, 0x7f].to_vec());
        let b = tc.consume_float();
        assert!(b.is_ok());
        assert!(b.unwrap().is_nan());

        reset_with_data(&mut tc, [1, 0, 0, 0, 0, 0, 0xf8, 0x7f].to_vec());
        let b = tc.consume_float();
        assert!(b.is_ok());
        assert!(b.unwrap().is_nan());

        reset_with_data(
            &mut tc,
            [0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0x7f].to_vec(),
        );
        let b = tc.consume_float();
        assert!(b.is_ok());
        assert!(b.unwrap().is_nan());

        reset_with_data(&mut tc, [1, 0, 0, 0, 0, 0, 0, 0].to_vec());
        let b = tc.consume_float();
        assert!(b.is_ok());
        assert_eq!(b.unwrap(), 5e-324);

        reset_with_data(
            &mut tc,
            [0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xef, 0x7f].to_vec(),
        );
        let b = tc.consume_float();
        assert!(b.is_ok());
        assert_eq!(b.unwrap(), f64::MAX);

        reset_with_data(&mut tc, [0xc0].to_vec());
        let b = tc.consume_float();
        assert!(b.is_ok());
        assert_eq!(b.unwrap(), -2.0);

        reset_with_data(&mut tc, [0].to_vec());
        let b = tc.consume_float();
        assert!(b.is_ok());
        assert_eq!(b.unwrap(), 0.0);

        reset_with_data(&mut tc, [0x37, 0x40].to_vec());
        let b = tc.consume_float();
        assert!(b.is_ok());
        assert_eq!(b.unwrap(), 1.4349296274686127e-42);

        reset_with_data(&mut tc, [0, 0x37, 0x40].to_vec());
        let b = tc.consume_float();
        assert!(b.is_ok());
        assert_eq!(b.unwrap(), 1.2933241802573108e-307);

        reset_with_data(&mut tc, [0, 0, 0x37, 0x40].to_vec());
        let b = tc.consume_float();
        assert!(b.is_ok());
        assert_eq!(b.unwrap(), 3.0013508467413e-310);

        reset_with_data(&mut tc, [0, 0, 0, 0x37, 0x40].to_vec());
        let b = tc.consume_float();
        assert!(b.is_ok());
        assert_eq!(b.unwrap(), 1.17240267451e-312);

        reset_with_data(&mut tc, [0, 0, 0, 0, 0x37, 0x40].to_vec());
        let b = tc.consume_float();
        assert!(b.is_ok());
        assert_eq!(b.unwrap(), 4.579697947e-315);

        reset_with_data(&mut tc, [0, 0, 0, 0, 0, 0x37, 0x40].to_vec());
        let b = tc.consume_float();
        assert!(b.is_ok());
        assert_eq!(b.unwrap(), 1.7889445e-317);
    }

    fn reset_with_data(tc: &mut TestCase, data: Vec<u8>) {
        tc.data = data;
        tc.size = tc.data.len();
        tc.data_ptr = 0;
    }

    fn setup() -> TestCase {
        let data = vec![
            0x8a, 0x19, 0x0d, 0x44, 0x37, 0x0d, 0x38, 0x5e, 0x9b, 0xaa, 0xf3, 0xda, 0xaa, 0x88,
            0xf2, 0x9b, 0x6c, 0xba, 0xbe, 0xb1, 0xf2, 0xcf, 0x13, 0xb8, 0xac, 0x1a, 0x7f, 0x1c,
            0xc9, 0x90, 0xd0, 0xd9, 0x5c, 0x42, 0xb3, 0xfd, 0xe3, 0x05, 0xa4, 0x03, 0x37, 0x49,
            0x50, 0x4b, 0xbc, 0x39, 0xa2, 0x09, 0x6c, 0x2f, 0xaf, 0xd1, 0xb5, 0x47, 0xbf, 0x92,
            0xbd, 0x79, 0xe5, 0xc5, 0x6e, 0x51, 0xa4, 0xed, 0xe9, 0xbd, 0x40, 0x4a, 0xfc, 0x25,
            0x7a, 0x27, 0xc8, 0x92, 0xf7, 0x30, 0xde, 0x40, 0x66, 0x66, 0xe8, 0x5f, 0x65, 0x39,
            0x7e, 0x9e, 0x80, 0x2b, 0x01, 0x71, 0x2a, 0xff, 0xd3, 0x0a, 0xac, 0x6e, 0x49, 0x32,
            0x79, 0x10, 0x6a, 0x6f, 0x97, 0x96, 0x70, 0x7e, 0x50, 0x65, 0xc9, 0x1d, 0xbd, 0x4e,
            0x17, 0x04, 0x1e, 0xba, 0x26, 0xac, 0x1f, 0xe3, 0x37, 0x1c, 0x15, 0x43, 0x60, 0x41,
            0x2a, 0x7c, 0xca, 0x70, 0xce, 0xab, 0x20, 0x24, 0xf8, 0xd9, 0x1f, 0x14, 0x7c, 0x5c,
            0xdd, 0x6f, 0xb3, 0xd7, 0x8b, 0x63, 0x10, 0xb7, 0xda, 0x99, 0xaf, 0x99, 0x01, 0x21,
            0xe6, 0xe1, 0x86, 0x27, 0xbe, 0x8d, 0xdf, 0x1e, 0xea, 0x80, 0x0b, 0x8a, 0x60, 0xc3,
            0x3a, 0x85, 0x33, 0x53, 0x59, 0xe1, 0xb5, 0xf1, 0x62, 0xa6, 0x7b, 0x24, 0x94, 0xe3,
            0x8c, 0x10, 0x93, 0xf8, 0x6e, 0xc2, 0x00, 0x91, 0x90, 0x0b, 0x5d, 0x52, 0x4f, 0x21,
            0xe3, 0x40, 0x3a, 0x6e, 0xb6, 0x32, 0x15, 0xdb, 0x5d, 0x01, 0x86, 0x63, 0x83, 0x24,
            0xc5, 0xde, 0xab, 0x31, 0x84, 0xaa, 0xe5, 0x64, 0x02, 0x8d, 0x23, 0x82, 0x86, 0x14,
            0x16, 0x18, 0x9f, 0x3d, 0x31, 0xbe, 0x3b, 0xf0, 0x6c, 0x26, 0x42, 0x9a, 0x67, 0xfe,
            0x28, 0xec, 0x28, 0xdb, 0x01, 0xb4, 0x52, 0x41, 0x81, 0x7c, 0x54, 0xd3, 0xc8, 0x00,
            0x01, 0x66, 0xb0, 0x2c, 0x3f, 0xbc, 0xaf, 0xac, 0x87, 0xcd, 0x83, 0xcf, 0x23, 0xfc,
            0xc8, 0x97, 0x8c, 0x71, 0x32, 0x8b, 0xbf, 0x70, 0xc0, 0x48, 0x31, 0x92, 0x18, 0xfe,
            0xe5, 0x33, 0x48, 0x82, 0x98, 0x1e, 0x30, 0xcc, 0xad, 0x5d, 0x97, 0xc4, 0xb4, 0x39,
            0x7c, 0xcd, 0x39, 0x44, 0xf1, 0xa9, 0xd0, 0xf4, 0x27, 0xb7, 0x78, 0x85, 0x9e, 0x72,
            0xfc, 0xcc, 0xee, 0x98, 0x25, 0x3b, 0x69, 0x6b, 0x0c, 0x11, 0xea, 0x22, 0xb6, 0xd0,
            0xcd, 0xbf, 0x6d, 0xbe, 0x12, 0xde, 0xfe, 0x78, 0x2e, 0x54, 0xcb, 0xba, 0xd7, 0x2e,
            0x54, 0x25, 0x14, 0x84, 0xfe, 0x1a, 0x10, 0xce, 0xcc, 0x20, 0xe6, 0xe2, 0x7f, 0xe0,
            0x5f, 0xdb, 0xa7, 0xf3, 0xe2, 0x4c, 0x52, 0x82, 0xfc, 0x0b, 0xa0, 0xbd, 0x34, 0x21,
            0xf7, 0xeb, 0x1c, 0x5b, 0x67, 0xd0, 0xaf, 0x22, 0x15, 0xa1, 0xff, 0xc2, 0x68, 0x25,
            0x5b, 0xb2, 0x13, 0x3f, 0xff, 0x98, 0x53, 0x25, 0xc5, 0x58, 0x39, 0xd0, 0x43, 0x86,
            0x6c, 0x5b, 0x57, 0x8e, 0x83, 0xba, 0xb9, 0x09, 0x09, 0x14, 0x0c, 0x9e, 0x99, 0x83,
            0x88, 0x53, 0x79, 0xfd, 0xf7, 0x49, 0xe9, 0x2c, 0xce, 0xe6, 0x7b, 0xf5, 0xc2, 0x27,
            0x5e, 0x56, 0xb5, 0xb4, 0x46, 0x90, 0x91, 0x7f, 0x99, 0x88, 0xa7, 0x23, 0xc1, 0x80,
            0xb8, 0x2d, 0xcd, 0xf7, 0x6f, 0x9a, 0xec, 0xbd, 0x16, 0x9f, 0x7d, 0x87, 0x1e, 0x15,
            0x51, 0xc4, 0x96, 0xe2, 0xbf, 0x61, 0x66, 0xb5, 0xfd, 0x01, 0x67, 0xd6, 0xff, 0xd2,
            0x14, 0x20, 0x98, 0x8e, 0xef, 0xf3, 0x22, 0xdb, 0x7e, 0xce, 0x70, 0x2d, 0x4c, 0x06,
            0x5a, 0xa0, 0x4f, 0xc8, 0xb0, 0x4d, 0xa6, 0x52, 0xb2, 0xd6, 0x2f, 0xd8, 0x57, 0xe5,
            0xef, 0xf9, 0xee, 0x52, 0x0f, 0xec, 0xc4, 0x90, 0x33, 0xad, 0x25, 0xda, 0xcd, 0x12,
            0x44, 0x5f, 0x32, 0xf6, 0x6f, 0xef, 0x85, 0xb8, 0xdc, 0x3c, 0x01, 0x48, 0x28, 0x5d,
            0x2d, 0x9c, 0x9b, 0xc0, 0x49, 0x36, 0x1e, 0x6a, 0x0a, 0x0c, 0xb0, 0x6e, 0x81, 0x89,
            0xcb, 0x0a, 0x89, 0xcf, 0x73, 0xc6, 0x63, 0x3d, 0x8e, 0x13, 0x57, 0x91, 0x4e, 0xa3,
            0x93, 0x8c, 0x61, 0x67, 0xfd, 0x13, 0xe0, 0x14, 0x72, 0xb3, 0xe4, 0x23, 0x45, 0x08,
            0x4e, 0x4e, 0xf5, 0xa7, 0xa8, 0xee, 0x30, 0xfd, 0x81, 0x80, 0x1f, 0xf3, 0x4f, 0xd7,
            0xe7, 0xf2, 0x16, 0xc0, 0xd6, 0x15, 0x6a, 0x0f, 0x89, 0x15, 0xa9, 0xcf, 0x35, 0x50,
            0x6b, 0x49, 0x3e, 0x12, 0x4a, 0x72, 0xe4, 0x59, 0x9d, 0xd7, 0xdb, 0xd2, 0xd1, 0x61,
            0x7d, 0x52, 0x4a, 0x36, 0xf6, 0xba, 0x0e, 0xfa, 0x88, 0x6f, 0x3c, 0x82, 0x16, 0xf0,
            0xd5, 0xed, 0x4d, 0x78, 0xef, 0x38, 0x17, 0x90, 0xea, 0x28, 0x32, 0xa9, 0x79, 0x40,
            0xff, 0xaa, 0xe6, 0xf5, 0xc7, 0x96, 0x56, 0x65, 0x61, 0x83, 0x3d, 0xbd, 0xd7, 0xed,
            0xd6, 0xb6, 0xc0, 0xed, 0x34, 0xaa, 0x60, 0xa9, 0xe8, 0x82, 0x78, 0xea, 0x69, 0xf6,
            0x47, 0xaf, 0x39, 0xab, 0x11, 0xdb, 0xe9, 0xfb, 0x68, 0x0c, 0xfe, 0xdf, 0x97, 0x9f,
            0x3a, 0xf4, 0xf3, 0x32, 0x27, 0x30, 0x57, 0x0e, 0xf7, 0xb2, 0xee, 0xfb, 0x1e, 0x98,
            0xa8, 0xa3, 0x25, 0x45, 0xe4, 0x6d, 0x2d, 0xae, 0xfe, 0xda, 0xb3, 0x32, 0x9b, 0x5d,
            0xf5, 0x32, 0x74, 0xea, 0xe5, 0x02, 0x30, 0x53, 0x95, 0x13, 0x7a, 0x23, 0x1f, 0x10,
            0x30, 0xea, 0x78, 0xe4, 0x36, 0x1d, 0x92, 0x96, 0xb9, 0x91, 0x2d, 0xfa, 0x43, 0xab,
            0xe6, 0xef, 0x14, 0x14, 0xc9, 0xbc, 0x46, 0xc6, 0x05, 0x7c, 0xc6, 0x11, 0x23, 0xcf,
            0x3d, 0xc8, 0xbe, 0xec, 0xa3, 0x58, 0x31, 0x55, 0x65, 0x14, 0xa7, 0x94, 0x93, 0xdd,
            0x2d, 0x76, 0xc9, 0x66, 0x06, 0xbd, 0xf5, 0xe7, 0x30, 0x65, 0x42, 0x52, 0xa2, 0x50,
            0x9b, 0xe6, 0x40, 0xa2, 0x4b, 0xec, 0xa6, 0xb7, 0x39, 0xaa, 0xd7, 0x61, 0x2c, 0xbf,
            0x37, 0x5a, 0xda, 0xb3, 0x5d, 0x2f, 0x5d, 0x11, 0x82, 0x97, 0x32, 0x8a, 0xc1, 0xa1,
            0x13, 0x20, 0x17, 0xbd, 0xa2, 0x91, 0x94, 0x2a, 0x4e, 0xbe, 0x3e, 0x77, 0x63, 0x67,
            0x5c, 0x0a, 0xe1, 0x22, 0x0a, 0x4f, 0x63, 0xe2, 0x84, 0xe9, 0x9f, 0x14, 0x86, 0xe2,
            0x4b, 0x20, 0x9f, 0x50, 0xb3, 0x56, 0xed, 0xde, 0x39, 0xd8, 0x75, 0x64, 0x45, 0x54,
            0xe5, 0x34, 0x57, 0x8c, 0x3b, 0xf2, 0x0e, 0x94, 0x1b, 0x10, 0xa2, 0xa2, 0x38, 0x76,
            0x21, 0x8e, 0x2a, 0x57, 0x64, 0x58, 0x0a, 0x27, 0x6d, 0x4c, 0xd0, 0xb5, 0xc1, 0xfc,
            0x75, 0xd0, 0x01, 0x86, 0x66, 0xa8, 0xf1, 0x98, 0x58, 0xfb, 0xfc, 0x64, 0xd2, 0x31,
            0x77, 0xad, 0x0e, 0x46, 0x87, 0xcc, 0x9b, 0x86, 0x90, 0xff, 0xb6, 0x64, 0x35, 0xa5,
            0x5d, 0x9e, 0x44, 0x51, 0x87, 0x9e, 0x1e, 0xee, 0xf3, 0x3b, 0x5c, 0xdd, 0x94, 0x03,
            0xaa, 0x18, 0x2c, 0xb7, 0xc4, 0x37, 0xd5, 0x53, 0x28, 0x60, 0xef, 0x77, 0xef, 0x3b,
            0x9e, 0xd2, 0xce, 0xe9, 0x53, 0x2d, 0xf5, 0x19, 0x7e, 0xbb, 0xb5, 0x46, 0xe2, 0xf7,
            0xd6, 0x4d, 0x6d, 0x5b, 0x81, 0x56, 0x6b, 0x12, 0x55, 0x63, 0xc3, 0xab, 0x08, 0xbb,
            0x2e, 0xd5, 0x11, 0xbc, 0x18, 0xcb, 0x8b, 0x12, 0x2e, 0x3e, 0x75, 0x32, 0x98, 0x8a,
            0xde, 0x3c, 0xea, 0x33, 0x46, 0xe7, 0x7a, 0xa5, 0x12, 0x09, 0x26, 0x7e, 0x7e, 0x03,
            0x4f, 0xfd, 0xc0, 0xfd, 0xea, 0x4f, 0x83, 0x85, 0x39, 0x62, 0xfb, 0xa2, 0x33, 0xd9,
            0x2d, 0xb1, 0x30, 0x6f, 0x88, 0xab, 0x61, 0xcb, 0x32, 0xeb, 0x30, 0xf9, 0x51, 0xf6,
            0x1f, 0x3a, 0x11, 0x4d, 0xfd, 0x54, 0xd6, 0x3d, 0x43, 0x73, 0x39, 0x16, 0xcf, 0x3d,
            0x29, 0x4a,
        ];
        let size = data.len();

        TestCase {
            data,
            size,
            data_ptr: 0,
            energy: 0,
            accessed: Vec::new(),
        }
    }
}
