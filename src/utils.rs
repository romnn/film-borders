pub fn set_panic_hook() {
    #[cfg(feature = "console_error_panic_hook")]
    console_error_panic_hook::set_once();
}

pub fn clamp<T: PartialOrd>(v: T, lower: T, upper: T) -> T {
    if v < lower {
        lower
    } else if v > upper {
        upper
    } else {
        v
    }
}

#[cfg(test)]
mod tests {
}
