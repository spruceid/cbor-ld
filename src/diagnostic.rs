use crate::CborValue;
use std::fmt;

pub struct Diagnostic<T>(pub T);

impl<T: DisplayDiagnostic> fmt::Display for Diagnostic<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt_diagnostic(f)
    }
}

pub trait DisplayDiagnostic {
    fn fmt_diagnostic(&self, f: &mut fmt::Formatter) -> fmt::Result;
}

impl<'a, T: DisplayDiagnostic> DisplayDiagnostic for &'a T {
    fn fmt_diagnostic(&self, f: &mut fmt::Formatter) -> fmt::Result {
        T::fmt_diagnostic(*self, f)
    }
}

impl<T: DisplayDiagnostic> DisplayDiagnostic for Box<T> {
    fn fmt_diagnostic(&self, f: &mut fmt::Formatter) -> fmt::Result {
        T::fmt_diagnostic(self, f)
    }
}

impl DisplayDiagnostic for CborValue {
    fn fmt_diagnostic(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::Null => write!(f, "null"),
            Self::Bool(true) => write!(f, "true"),
            Self::Bool(false) => write!(f, "false"),
            Self::Integer(i) => {
                let i: i128 = i.clone().into();
                write!(f, "{i}")
            }
            Self::Float(v) => {
                write!(f, "{v}")
            }
            Self::Text(s) => {
                write!(f, "\"{s}\"")
            }
            Self::Tag(t, v) => {
                write!(f, "{t}({})", Diagnostic(v))
            }
            Self::Array(array) => {
                write!(f, "[")?;

                for (i, item) in array.iter().enumerate() {
                    if i > 0 {
                        write!(f, ",")?;
                    }

                    item.fmt_diagnostic(f)?
                }

                write!(f, "]")
            }
            Self::Map(entries) => {
                write!(f, "[")?;

                for (i, (key, value)) in entries.iter().enumerate() {
                    if i > 0 {
                        write!(f, ",")?;
                    }

                    write!(f, "{}=>{}", Diagnostic(key), Diagnostic(value))?
                }

                write!(f, "]")
            }
            Self::Bytes(bytes) => {
                write!(f, "h'")?;

                for b in bytes {
                    write!(f, "{b:#}")?;
                }

                write!(f, "'")
            }
            _ => panic!(),
        }
    }
}
