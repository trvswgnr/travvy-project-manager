use std::{io, error::Error, fmt, ffi::OsString};

pub enum DynErr {
    String(String),
    Io(io::Error),
    Serde(serde_json::Error),
    Std(Box<dyn Error>),
}

impl From<String> for DynErr {
    fn from(err: String) -> Self {
        DynErr::String(err)
    }
}

impl From<dialoguer::Error> for DynErr {
    fn from(err: dialoguer::Error) -> Self {
        DynErr::String(err.to_string())
    }
}

impl<T: fmt::Display> From<Option<T>> for DynErr {
    fn from(err: Option<T>) -> Self {
        match err {
            Some(err) => DynErr::String(err.to_string()),
            None => DynErr::String("".to_string()),
        }
    }
}

impl From<&str> for DynErr {
    fn from(err: &str) -> Self {
        DynErr::String(err.to_string())
    }
}

impl From<OsString> for DynErr {
    fn from(err: OsString) -> Self {
        DynErr::String(
            err.into_string()
                .unwrap_or_else(|_| "Problem converting OsString to String".into()),
        )
    }
}

impl From<io::Error> for DynErr {
    fn from(err: io::Error) -> Self {
        DynErr::Io(err)
    }
}

impl From<serde_json::Error> for DynErr {
    fn from(err: serde_json::Error) -> Self {
        DynErr::Serde(err)
    }
}

impl<T: 'static> From<std::sync::PoisonError<T>> for DynErr {
    fn from(err: std::sync::PoisonError<T>) -> Self {
        DynErr::Std(Box::new(err))
    }
}

impl From<Box<dyn Error>> for DynErr {
    fn from(err: Box<dyn Error>) -> Self {
        DynErr::Std(err)
    }
}

impl From<std::time::SystemTimeError> for DynErr {
    fn from(err: std::time::SystemTimeError) -> Self {
        DynErr::Std(Box::new(err))
    }
}

impl fmt::Display for DynErr {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            DynErr::String(err) => write!(f, "{}", err),
            DynErr::Io(err) => write!(f, "{}", err),
            DynErr::Serde(err) => write!(f, "{}", err),
            DynErr::Std(err) => write!(f, "{}", err),
        }
    }
}
