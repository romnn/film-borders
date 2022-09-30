use pretty_assertions::StrComparison;
use std::fmt;

pub fn assert_failed_pretty(
    left: &dyn fmt::Debug,
    op: &str,
    right: &dyn fmt::Debug,
    args: Option<fmt::Arguments<'_>>,
) -> ! {
    let left = format!("{:?}", left);
    let right = format!("{:?}", right);
    let diff = StrComparison::new(&left, &right);
    match args {
        Some(args) => panic!(
            r#"assertion failed: `(left {} right)`
`{}`
`{:?}`"#,
            op, diff, args
        ),
        None => panic!(
            r#"assertion failed: `(left {} right)`
 `{}`"#,
            op, diff
        ),
    }
}

#[allow(dead_code)]
pub fn assert_failed(
    left: &dyn fmt::Debug,
    op: &str,
    right: &dyn fmt::Debug,
    args: Option<fmt::Arguments<'_>>,
) -> ! {
    match args {
        Some(args) => panic!(
            r#"assertion failed: `(left {} right)`
    left: `{:?}`,
    right: `{:?}`: `{:?}`"#,
            op, left, right, args
        ),
        None => panic!(
            r#"assertion failed: `(left {} right)`
    left: `{:?}`,
    right: `{:?}`"#,
            op, left, right
        ),
    }
}

macro_rules! assert_matches_regex {
    ($value:expr, $( $regex:expr ),* $(,)? ) => {{
        $(
        match (&$value, &$regex) {
            (value, regex) => {
                if !regex::Regex::new(regex).unwrap().is_match(value) {
                    $crate::test::assert_failed_pretty(value, "matches", regex, std::option::Option::None);
                }
            }
        }
        )*
    }};
}

pub(crate) use assert_matches_regex;
