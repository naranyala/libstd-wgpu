/// Load WGSL from a path relative to the crate root.
#[macro_export]
macro_rules! include_wgsl {
    ($path:literal) => {
        include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/", $path))
    };
}
