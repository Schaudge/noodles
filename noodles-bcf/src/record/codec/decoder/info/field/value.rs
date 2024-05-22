use std::{error, fmt};

use noodles_vcf::{self as vcf, header::record::value::map::info::Type};

use crate::record::codec::{
    decoder::value,
    value::{Array, Float, Int16, Int32, Int8},
    Value,
};

pub(super) fn read_value(
    src: &mut &[u8],
    ty: Type,
) -> Result<Option<vcf::variant::record_buf::info::field::Value>, DecodeError> {
    match ty {
        Type::Integer => read_integer_value(src),
        Type::Flag => read_flag_value(src),
        Type::Float => read_float_value(src),
        Type::Character => read_character_value(src),
        Type::String => read_string_value(src),
    }
}

fn read_integer_value(
    src: &mut &[u8],
) -> Result<Option<vcf::variant::record_buf::info::field::Value>, DecodeError> {
    match value::read_value(src).map_err(DecodeError::InvalidValue)? {
        None
        | Some(Value::Int8(None | Some(Int8::Missing)))
        | Some(Value::Int16(None | Some(Int16::Missing)))
        | Some(Value::Int32(None | Some(Int32::Missing))) => Ok(None),
        Some(Value::Int8(Some(Int8::Value(n)))) => Ok(Some(
            vcf::variant::record_buf::info::field::Value::from(i32::from(n)),
        )),
        Some(Value::Array(Array::Int8(values))) => {
            Ok(Some(vcf::variant::record_buf::info::field::Value::from(
                values
                    .iter()
                    .map(|result| {
                        result.map(Int8::from).map(|value| match value {
                            Int8::Value(n) => Some(i32::from(n)),
                            Int8::Missing => None,
                            _ => todo!("unhandled i8 array value: {:?}", value),
                        })
                    })
                    .collect::<Result<Vec<_>, _>>()
                    .map_err(|_| DecodeError::UnexpectedEof)?,
            )))
        }
        Some(Value::Int16(Some(Int16::Value(n)))) => Ok(Some(
            vcf::variant::record_buf::info::field::Value::from(i32::from(n)),
        )),
        Some(Value::Array(Array::Int16(values))) => {
            Ok(Some(vcf::variant::record_buf::info::field::Value::from(
                values
                    .iter()
                    .map(|result| {
                        result.map(Int16::from).map(|value| match value {
                            Int16::Value(n) => Some(i32::from(n)),
                            Int16::Missing => None,
                            _ => todo!("unhandled i16 array value: {:?}", value),
                        })
                    })
                    .collect::<Result<Vec<_>, _>>()
                    .map_err(|_| DecodeError::UnexpectedEof)?,
            )))
        }
        Some(Value::Int32(Some(Int32::Value(n)))) => {
            Ok(Some(vcf::variant::record_buf::info::field::Value::from(n)))
        }
        Some(Value::Array(Array::Int32(values))) => {
            Ok(Some(vcf::variant::record_buf::info::field::Value::from(
                values
                    .iter()
                    .map(|result| {
                        result.map(Int32::from).map(|value| match value {
                            Int32::Value(n) => Some(n),
                            Int32::Missing => None,
                            _ => todo!("unhandled i32 array value: {:?}", value),
                        })
                    })
                    .collect::<Result<Vec<_>, _>>()
                    .map_err(|_| DecodeError::UnexpectedEof)?,
            )))
        }
        v => Err(type_mismatch_error(v, Type::Integer)),
    }
}

fn read_flag_value(
    src: &mut &[u8],
) -> Result<Option<vcf::variant::record_buf::info::field::Value>, DecodeError> {
    match value::read_value(src).map_err(DecodeError::InvalidValue)? {
        None | Some(Value::Int8(Some(Int8::Value(1)))) => {
            Ok(Some(vcf::variant::record_buf::info::field::Value::Flag))
        }
        v => Err(type_mismatch_error(v, Type::Flag)),
    }
}

fn read_float_value(
    src: &mut &[u8],
) -> Result<Option<vcf::variant::record_buf::info::field::Value>, DecodeError> {
    match value::read_value(src).map_err(DecodeError::InvalidValue)? {
        None | Some(Value::Float(None | Some(Float::Missing))) => Ok(None),
        Some(Value::Float(Some(Float::Value(n)))) => {
            Ok(Some(vcf::variant::record_buf::info::field::Value::from(n)))
        }
        Some(Value::Array(Array::Float(values))) => {
            Ok(Some(vcf::variant::record_buf::info::field::Value::from(
                values
                    .iter()
                    .map(|result| {
                        result.map(Float::from).map(|value| match value {
                            Float::Value(n) => Some(n),
                            Float::Missing => None,
                            _ => todo!("unhandled float array value: {:?}", value),
                        })
                    })
                    .collect::<Result<Vec<_>, _>>()
                    .map_err(|_| DecodeError::UnexpectedEof)?,
            )))
        }
        v => Err(type_mismatch_error(v, Type::Float)),
    }
}

fn read_character_value(
    src: &mut &[u8],
) -> Result<Option<vcf::variant::record_buf::info::field::Value>, DecodeError> {
    const DELIMITER: char = ',';
    const MISSING_VALUE: char = '.';

    match value::read_value(src).map_err(DecodeError::InvalidValue)? {
        None | Some(Value::String(None)) => Ok(None),
        Some(Value::String(Some(s))) => match s.len() {
            0 | 1 => s
                .chars()
                .next()
                .map(vcf::variant::record_buf::info::field::Value::from)
                .map(|v| Ok(Some(v)))
                .ok_or(DecodeError::MissingCharacter)?,
            _ => Ok(Some(vcf::variant::record_buf::info::field::Value::from(
                s.split(DELIMITER)
                    .flat_map(|t| t.chars())
                    .map(|c| match c {
                        MISSING_VALUE => None,
                        _ => Some(c),
                    })
                    .collect::<Vec<_>>(),
            ))),
        },
        v => Err(type_mismatch_error(v, Type::Character)),
    }
}

fn read_string_value(
    src: &mut &[u8],
) -> Result<Option<vcf::variant::record_buf::info::field::Value>, DecodeError> {
    match value::read_value(src).map_err(DecodeError::InvalidValue)? {
        None | Some(Value::String(None)) => Ok(None),
        Some(Value::String(Some(s))) => {
            Ok(Some(vcf::variant::record_buf::info::field::Value::from(s)))
        }
        v => Err(type_mismatch_error(v, Type::String)),
    }
}

fn type_mismatch_error(value: Option<Value>, expected: Type) -> DecodeError {
    let actual = value.map(|v| match v {
        Value::Int8(_) | Value::Int16(_) | Value::Int32(_) => Type::Integer,
        Value::Float(_) => Type::Float,
        Value::String(_) => Type::String,
        Value::Array(Array::Int8(_) | Array::Int16(_) | Array::Int32(_)) => Type::Integer,
        Value::Array(Array::Float(_)) => Type::Float,
    });

    DecodeError::TypeMismatch { actual, expected }
}

#[derive(Debug, Eq, PartialEq)]
pub enum DecodeError {
    UnexpectedEof,
    InvalidValue(value::DecodeError),
    TypeMismatch {
        actual: Option<Type>,
        expected: Type,
    },
    MissingCharacter,
}

impl error::Error for DecodeError {
    fn source(&self) -> Option<&(dyn error::Error + 'static)> {
        match self {
            Self::InvalidValue(e) => Some(e),
            _ => None,
        }
    }
}

impl fmt::Display for DecodeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::UnexpectedEof => write!(f, "unexpected EOF"),
            Self::InvalidValue(_) => write!(f, "invalid value"),
            Self::TypeMismatch { actual, expected } => {
                write!(f, "type mismatch: expected {expected:?}, got {actual:?}")
            }
            Self::MissingCharacter => write!(f, "missing character"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_read_value_with_integer_value() {
        fn t(mut src: &[u8], expected_value: Option<i32>) {
            let actual = read_value(&mut src, Type::Integer);
            let expected = expected_value.map(vcf::variant::record_buf::info::field::Value::from);
            assert_eq!(actual, Ok(expected));
        }

        // None
        t(&[0x00], None);

        // Some(Value::Int8(None))
        t(&[0x01], None);
        // Some(Value::Int8(Int8::Missing))
        t(&[0x11, 0x80], None);
        // Some(Value::Int8(Some(Int8::Value(8))))
        t(&[0x11, 0x08], Some(8));

        // Some(Value::Int16(None))
        t(&[0x02], None);
        // Some(Value::Int16(Int16::Missing))
        t(&[0x12, 0x00, 0x80], None);
        // Some(Value::Int16(Some(Int16::Value(13))))
        t(&[0x12, 0x0d, 0x00], Some(13));

        // Some(Value::Int32(None))
        t(&[0x03], None);
        // Some(Value::Int32(Int32::Missing))
        t(&[0x13, 0x00, 0x00, 0x00, 0x80], None);
        // Some(Value::Int32(Some(Int32::Value(21))))
        t(&[0x13, 0x15, 0x00, 0x00, 0x00], Some(21));
    }

    #[test]
    fn test_read_value_with_integer_array_value() {
        fn t(mut src: &[u8], expected_value: Option<Vec<Option<i32>>>) {
            let actual = read_value(&mut src, Type::Integer);
            let expected = expected_value.map(vcf::variant::record_buf::info::field::Value::from);
            assert_eq!(actual, Ok(expected));
        }

        // Some(Value::IntegerArray([Some(8), Some(13)]))
        t(&[0x21, 0x08, 0x0d], Some(vec![Some(8), Some(13)]));
        // Some(Value::IntegerArray([Some(8), None]))
        t(&[0x21, 0x08, 0x80], Some(vec![Some(8), None]));

        // Some(Value::IntegerArray([Some(21), Some(34)]))
        t(
            &[0x22, 0x15, 0x00, 0x22, 0x00],
            Some(vec![Some(21), Some(34)]),
        );
        // Some(Value::IntegerArray([Some(21), None]))
        t(&[0x22, 0x15, 0x00, 0x00, 0x80], Some(vec![Some(21), None]));

        // Some(Value::IntegerArray([Some(55), Some(89)]))
        t(
            &[0x23, 0x37, 0x00, 0x00, 0x00, 0x59, 0x00, 0x00, 0x00],
            Some(vec![Some(55), Some(89)]),
        );
        // Some(Value::IntegerArray([Some(55), None]))
        t(
            &[0x23, 0x37, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x80],
            Some(vec![Some(55), None]),
        );
    }

    #[test]
    fn test_read_value_with_flag_value() {
        fn t(mut src: &[u8]) {
            let actual = read_value(&mut src, Type::Flag);
            let expected = Some(vcf::variant::record_buf::info::field::Value::Flag);
            assert_eq!(actual, Ok(expected));
        }

        // None
        t(&[0x00]);
        // Some(Value::Int8(Some(Int8::Value(1))))
        t(&[0x11, 0x01]);
    }

    #[test]
    fn test_read_value_with_float_value() {
        fn t(mut src: &[u8], expected_value: Option<f32>) {
            let actual = read_value(&mut src, Type::Float);
            let expected = expected_value.map(vcf::variant::record_buf::info::field::Value::from);
            assert_eq!(actual, Ok(expected));
        }

        // None
        t(&[0x00], None);
        // Some(Value::Float(None))
        t(&[0x05], None);
        // Some(Value::Float(Some(Float::Missing)))
        t(&[0x15, 0x01, 0x00, 0x80, 0x7f], None);

        // Some(Value::Float(Some(Float::Value(0.0))))
        t(&[0x15, 0x00, 0x00, 0x00, 0x00], Some(0.0));
    }

    #[test]
    fn test_read_value_with_float_array_value() {
        fn t(mut src: &[u8], expected_value: Option<Vec<Option<f32>>>) {
            let actual = read_value(&mut src, Type::Float);
            let expected = expected_value.map(vcf::variant::record_buf::info::field::Value::from);
            assert_eq!(actual, Ok(expected));
        }

        // Some(Value::FloatArray([0.0, 1.0]))
        t(
            &[0x25, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x80, 0x3f],
            Some(vec![Some(0.0), Some(1.0)]),
        );
        // Some(Value::FloatArray([0.0, None]))
        t(
            &[0x25, 0x00, 0x00, 0x00, 0x00, 0x01, 0x00, 0x80, 0x7f],
            Some(vec![Some(0.0), None]),
        );
    }

    #[test]
    fn test_read_value_with_character_value() {
        fn t(mut src: &[u8], expected_value: Option<char>) {
            let actual = read_value(&mut src, Type::Character);
            let expected = expected_value.map(vcf::variant::record_buf::info::field::Value::from);
            assert_eq!(actual, Ok(expected));
        }

        // None
        t(&[0x00], None);
        // Some(Value::String(None))
        t(&[0x07], None);

        // Some(Value::String(Some(String::from("n"))))
        t(&[0x17, 0x6e], Some('n'));
    }

    #[test]
    fn test_read_value_with_character_array_value() {
        fn t(mut src: &[u8], expected_value: Option<Vec<Option<char>>>) {
            let actual = read_value(&mut src, Type::Character);
            let expected = expected_value.map(vcf::variant::record_buf::info::field::Value::from);
            assert_eq!(actual, Ok(expected));
        }

        // None
        t(&[0x00], None);

        // Some(Value::String(Some(String::from("n,d"))))
        t(&[0x37, 0x6e, 0x2c, 0x64], Some(vec![Some('n'), Some('d')]));
        // Some(Value::String(Some(String::from("n,."))))
        t(&[0x37, 0x6e, 0x2c, 0x2e], Some(vec![Some('n'), None]));
    }

    #[test]
    fn test_read_value_with_string_value() {
        fn t(mut src: &[u8], expected_value: Option<&str>) {
            let actual = read_value(&mut src, Type::String);
            let expected = expected_value.map(vcf::variant::record_buf::info::field::Value::from);
            assert_eq!(actual, Ok(expected));
        }

        // None
        t(&[0x00], None);

        // Some(Value::String(None))
        t(&[0x07], None);
        // Some(Value::String(Some(String::from("ndls"))))
        t(&[0x47, 0x6e, 0x64, 0x6c, 0x73], Some("ndls"));
    }
}
