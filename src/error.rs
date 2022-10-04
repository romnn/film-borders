use std::fmt::Write;

pub trait Report {
    fn report(&self) -> String;
}

impl<E> Report for E
where
    E: std::error::Error,
{
    fn report(&self) -> String {
        let mut buf = String::new();
        writeln!(buf, "ERROR: {}", self).ok();
        if let Some(cause) = self.source() {
            writeln!(buf).ok();
            writeln!(buf, "Caused by:").ok();
            let causes = std::iter::successors(Some(cause), |e| e.source());
            for (i, e) in causes.enumerate() {
                writeln!(buf, "   {}: {}", i, e).ok();
            }
        }
        buf
    }
}

#[derive(thiserror::Error, Debug)]
pub enum ParseEnum {
    #[error("unknown enum variant: `{0}`")]
    Unknown(String),
}
