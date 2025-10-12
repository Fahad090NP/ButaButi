//! Macros for binary reading and writing with better error context
//!
//! Provides convenient macros for common binary I/O operations with automatic
//! error handling and context information.

/// Read magic bytes and validate they match expected value
///
/// # Example
///
/// ```rust,ignore
/// use butabuti::read_magic;
///
/// read_magic!(reader, b"DST")?;  // Validates DST header magic
/// ```
#[macro_export]
macro_rules! read_magic {
    ($reader:expr, $expected:expr) => {{
        let expected = $expected;
        let mut actual = vec![0u8; expected.len()];
        $reader
            .read_exact(&mut actual)
            .map_err(|e| $crate::utils::error::Error::Io(e))?;
        if actual != expected {
            return Err($crate::utils::error::Error::Parse(format!(
                "Magic bytes mismatch at {}:{}. Expected {:?}, got {:?}",
                file!(),
                line!(),
                expected,
                actual
            )));
        }
        actual
    }};
}

/// Read an exact number of bytes with context
///
/// # Example
///
/// ```rust,ignore
/// use butabuti::read_bytes;
///
/// let header = read_bytes!(reader, 512, "reading DST header")?;
/// ```
#[macro_export]
macro_rules! read_bytes {
    ($reader:expr, $count:expr) => {{
        let mut buffer = vec![0u8; $count];
        $reader.read_exact(&mut buffer).map_err(|e| {
            if e.kind() == std::io::ErrorKind::UnexpectedEof {
                $crate::utils::error::Error::Parse(format!(
                    "Unexpected EOF reading {} bytes at {}:{}",
                    $count,
                    file!(),
                    line!()
                ))
            } else {
                $crate::utils::error::Error::Io(e)
            }
        })?;
        buffer
    }};
    ($reader:expr, $count:expr, $context:expr) => {{
        let mut buffer = vec![0u8; $count];
        $reader.read_exact(&mut buffer).map_err(|e| {
            if e.kind() == std::io::ErrorKind::UnexpectedEof {
                $crate::utils::error::Error::Parse(format!(
                    "Unexpected EOF while {}: {} bytes at {}:{}",
                    $context,
                    $count,
                    file!(),
                    line!()
                ))
            } else {
                $crate::utils::error::Error::Io(e)
            }
        })?;
        buffer
    }};
}

/// Wrap a read operation with context for better error messages
///
/// # Example
///
/// ```rust,ignore
/// use butabuti::read_with_context;
///
/// let value = read_with_context!(
///     reader.read_u16::<LittleEndian>(),
///     "reading stitch count"
/// )?;
/// ```
#[macro_export]
macro_rules! read_with_context {
    ($operation:expr, $context:expr) => {{
        $operation.map_err(|e| {
            if e.kind() == std::io::ErrorKind::UnexpectedEof {
                $crate::utils::error::Error::Parse(format!(
                    "Unexpected EOF while {} at {}:{}",
                    $context,
                    file!(),
                    line!()
                ))
            } else {
                $crate::utils::error::Error::Io(e)
            }
        })?
    }};
}

#[cfg(test)]
mod tests {
    use crate::utils::error::Result;
    use std::io::{Cursor, Read};

    #[test]
    fn test_read_magic_success() -> Result<()> {
        let data = b"DST\x1a";
        let mut reader = Cursor::new(&data[..]);
        let _result = read_magic!(reader, b"DST\x1a");
        Ok(())
    }

    #[test]
    fn test_read_magic_mismatch() {
        fn inner() -> Result<()> {
            let data = b"PES\x00";
            let mut reader = Cursor::new(&data[..]);
            let _result = read_magic!(reader, b"DST\x1a");
            Ok(())
        }
        assert!(inner().is_err());
    }

    #[test]
    fn test_read_bytes_success() -> Result<()> {
        let data = vec![1, 2, 3, 4, 5];
        let mut reader = Cursor::new(data);
        let result = read_bytes!(reader, 5);
        assert_eq!(result, vec![1, 2, 3, 4, 5]);
        Ok(())
    }

    #[test]
    fn test_read_bytes_eof() {
        fn inner() -> Result<()> {
            let data = vec![1, 2, 3];
            let mut reader = Cursor::new(data);
            let _result = read_bytes!(reader, 5);
            Ok(())
        }
        assert!(inner().is_err());
    }

    #[test]
    fn test_read_bytes_with_context() -> Result<()> {
        let data = vec![1, 2, 3, 4];
        let mut reader = Cursor::new(data);
        let result = read_bytes!(reader, 4, "test context");
        assert_eq!(result, vec![1, 2, 3, 4]);
        Ok(())
    }
}
