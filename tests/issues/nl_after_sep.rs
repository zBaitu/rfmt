
impl fmt::Display for DeserializeError {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            DeserializeError::IoError(ref ioerr) =>
                write!(fmt, "IoError: {}", ioerr),
            DeserializeError::InvalidEncoding(ref ib) =>
                write!(fmt, "InvalidEncoding: {}", ib),
            DeserializeError::SizeLimit =>
                write!(fmt, "SizeLimit"),
            DeserializeError::SyntaxError =>
                write!(fmt, "SyntaxError"),
            DeserializeError::EndOfStreamError =>
                write!(fmt, "EndOfStreamError"),
            DeserializeError::UnknownFieldError =>
                write!(fmt, "UnknownFieldError"),
            DeserializeError::MissingFieldError =>
                write!(fmt, "MissingFieldError"),
        }
    }
}

