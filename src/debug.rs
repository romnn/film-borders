use serde::Serialize;
use wasm_bindgen::{JsError, JsValue};

// not monotonic but the best thing i found that works with wasm
pub struct Instant(chrono::NaiveTime);

impl std::ops::Deref for Instant {
    type Target = chrono::NaiveTime;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

pub struct Duration(chrono::Duration);

impl std::ops::Deref for Duration {
    type Target = chrono::Duration;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl std::fmt::Display for Duration {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        use duration_string::DurationString;
        let duration = self
            .0
            .to_std()
            .ok()
            .map(|duration| DurationString::from(duration).to_string())
            .unwrap_or("?".to_string());
        write!(f, "{}", duration)
    }
}

impl Instant {
    pub fn now() -> Self {
        Self(chrono::Utc::now().time())
    }

    pub fn elapsed(&self) -> Duration {
        let now = Self::now();
        let duration = *now - **self;
        Duration(duration)
    }

    pub fn elapsed_millis(&self) -> i64 {
        self.elapsed().num_milliseconds()
    }
}

pub trait AsJson {
    fn into_json(self) -> Result<JsValue, JsValue>;
}

impl<T> AsJson for T
where
    T: Serialize,
{
    fn into_json(self) -> Result<JsValue, JsValue> {
        let json = serde_json::to_string(&self).map_err(JsError::from)?;
        let json = js_sys::JSON::parse(&json)?;
        Ok(json)
    }
}

#[cfg(feature = "debug")]
#[macro_export]
macro_rules! debug {
    ($val:literal $(,)?) => {{
        #[cfg(target_arch = "wasm32")]
        web_sys::console::log(&js_sys::Array::from_iter([
            &match $crate::debug::AsJson::into_json($val) {
                Ok(json) => json,
                Err(err) => err,
            }
        ]));
        #[cfg(not(target_arch = "wasm32"))]
        eprintln!("[{}:{}] {}", file!(), line!(), &$val);
    }};
    ($val:expr $(,)?) => {{
        match $val {
            tmp => {
                #[cfg(target_arch = "wasm32")]
                {
                    use $crate::debug::AsJson;
                    let values = js_sys::Array::from_iter([
                        &format!(
                            "[{}:{}] {} =",
                            file!(), line!(), stringify!($val)
                        ).into(),
                        &match $val.clone().into_json() {
                            Ok(json) => json,
                            Err(err) => err,
                        }
                    ]);
                    web_sys::console::log(&values);
                }
                #[cfg(not(target_arch = "wasm32"))]
                eprintln!("[{}:{}] {} = {:#?}",
                    file!(), line!(), stringify!($val), &tmp);
            }
        }
    }};
    ( $( $t:expr ),* $(,)? ) => {{
        #[cfg(target_arch = "wasm32")]
        {
            use $crate::debug::AsJson;
            let mut values = js_sys::Array::new();
            values.push(&format!("[{}:{}]", file!(), line!()).into());
            $(
                match $t.clone().into_json() {
                    Ok(json) => values.push(&json),
                    Err(err) => values.push(&err),
                };
            )*
            web_sys::console::log(&values);
        }
        #[cfg(not(target_arch = "wasm32"))]
        {
            eprint!("[{}:{}]", file!(), line!());
            $(
                eprint!(" {}", $t);
            )*
            eprintln!("");
        }
    }};
}

#[cfg(not(feature = "debug"))]
#[macro_export]
macro_rules! debug {
    ($x:expr) => {{}};
    ($($arg:tt)*) => {};
}
