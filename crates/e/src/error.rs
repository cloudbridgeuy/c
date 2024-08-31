#[derive(thiserror::Error)]
pub enum Error {
    #[error("io error")]
    Io(#[from] std::io::Error),
    #[error("unable to stream the api")]
    EsStream(#[from] es_stream::error::Error),
    #[error("unable to get value from environment variable")]
    EnvVar(#[from] std::env::VarError),
    #[error("invalid api")]
    InvalidAPI,
}

pub(crate) fn format_error(
    e: &impl std::error::Error,
    f: &mut std::fmt::Formatter,
) -> std::fmt::Result {
    write!(f, "error: {e}")?;

    let mut source = e.source();

    if e.source().is_some() {
        writeln!(f, "\ncaused by:")?;
        let mut i: usize = 0;
        while let Some(inner) = source {
            writeln!(f, "{i: >5}: {inner}")?;
            source = inner.source();
            i += 1;
        }
    }

    Ok(())
}

impl std::fmt::Debug for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        format_error(self, f)
    }
}