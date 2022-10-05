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
        // static zero = std::time::Duration::ZERO;
        &self.0 // .unwrap_or(std::time::Duration::ZERO);
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
        // Some(duration) => write!(f, "{}", self.0)
        // .unwrap_or(std::time::Duration::ZERO)
    }
}

// DurationString::from(Duration::from_millis(100)).into()

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

// pub trait AsJson: Into<JsValue> {
//     fn into_json(self) -> Result<JsValue, JsValue> {
//         Ok(self.into())
//     }
// }

// impl<T> AsJson for T where T: Into<JsValue> {}

// impl AsJson for String {}
// impl AsJson for u32 {}
// impl AsJson for &str {}

pub trait Private: Into<JsValue> + Serialize {}

impl<T> AsJson for T
where
    T: Serialize,
{
    fn into_json(self) -> Result<JsValue, JsValue> {
        // let json = serde_json::to_string(&self).and_then(|json| js_sys::JSON::parse(json))?;
        let json = serde_json::to_string(&self).map_err(JsError::from)?;
        let json = js_sys::JSON::parse(&json)?;
        Ok(json)
        // let value = self.into();
        // let json = json.map_err(|_| value.clone())?;
        // js_sys::JSON::parse(&json).map_err(|_| value.clone())
        // Ok(json)
        // Ok(self.into())
        // Err(JsError::new("test"))
    }
}

// trait Private {}
// impl Private for String {}

// trait Other {}
// impl Other for u32 {}

// impl<T> AsJson for T
// where
//     T: Private, // T: TryInto<JsValue>,
//                   // <T as TryInto>::Error = JsError,
// {
//     fn into_json(self) -> Result<JsValue, JsError> {
//         // let json = serde_json::to_string(&self)?;
//         // let json = js_sys::JSON::parse(json)?;
//         // Ok(json)
//         // Ok(self.into())
//         Err(JsError::new("test"))
//     }
// }

// impl<T> AsJson for T
// where
//     T: Other, // T: TryInto<JsValue>,
//                   // <T as TryInto>::Error = JsError,
// {
//     fn into_json(self) -> Result<JsValue, JsError> {
//         // let json = serde_json::to_string(&self)?;
//         // let json = js_sys::JSON::parse(json)?;
//         // Ok(json)
//         // Ok(self.into())
//         Err(JsError::new("test"))
//     }
// }

// trait Native: Into<JsValue> {}

// impl<T> AsJson for T
// where
//     T: Native, // <T as TryInto>::Error = JsError,
// {
//     fn into_json(self) -> Result<JsValue, JsError> {
//         // let json = serde_json::to_string(&self)?;
//         // let json = js_sys::JSON::parse(json)?;
//         // Ok(json)
//         Ok(self.into())
//     }
// }

// Into<JsValue> +
// impl<T> AsJson for T where T: wasm_bindgen::JsCast + Into<JsValue> { //  {
// impl<T> AsJson for T
// where
//     T: Into<JsValue>,
//     // T: TryInto<JsValue>,
//     // <T as TryInto>::Error = JsError,
// {
//     fn into_json(self) -> Result<JsValue, JsError> {
//         Ok(self.into())
//     }
// }

// impl<T> AsJson for T
// where
//     // T: Into<JsValue>,
//     T: TryInto<JsValue>,
//     <T as TryInto<JsValue>>::Error: std::error::Error + Into<JsError>,
// {
//     fn into_json(self) -> Result<JsValue, JsError> {
//         let value = self.try_into()?;
//         Ok(value)
//     }
// }
// ($val:expr $(,)?) => {
// #[cfg(not(debug_assertions))]

#[cfg(feature = "debug")]
#[macro_export]
macro_rules! debug {
    // () => {
    //     #[cfg(target_arch = "wasm32")]
    //     super::wasm::console_log!("[{}:{}]", file!(), line!())
    //     {
    //     }
    //     #[cfg(not(target_arch = "wasm32"))]
    //     eprintln!("[{}:{}]", file!(), line!())
    // };
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
    // ( $( $t:tt )* ) => {{
        #[cfg(target_arch = "wasm32")]
        {
            use $crate::debug::AsJson;
            let mut values = js_sys::Array::new();
            values.push(&format!("[{}:{}]", file!(), line!()).into());
            $(
                // let value: wasm_bindgen::JsValue = $t.clone().into();
                // match js_sys::JSON::parse(value) {
                // let json = serde_json::to_string(&self)?;
                // let json = js_sys::JSON::parse(json)?;
                // Ok(json)

                // match js_sys::JSON::stringify(&value) {
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
                eprint!(" ");
                eprint!("{}", $t);
                eprint!(" ");
            )*
            eprintln!("");
        }
        // super::wasm::console_log!("[{}:{}]", file!(), line!())
        // match $val {
        //     tmp => {
        //         eprintln!("[{}:{}] {} = {:#?}",
        //             file!(), line!(), stringify!($val), &tmp);
        //         tmp
        //     }
        // }
    }};
    // ($($arg:tt)*) => {{
    //     println!($($arg)*);
    // }};
}

#[cfg(not(feature = "debug"))]
#[macro_export]
macro_rules! debug {
    ($x:expr) => {{
        // std::convert::identity($x)
    }};
    ($($arg:tt)*) => {};
}
