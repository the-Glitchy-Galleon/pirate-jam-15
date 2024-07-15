cfg_if::cfg_if! {
    if #[cfg(target_family = "wasm")] {
        mod wasm;
        pub use wasm::*;
    } else {
        mod desktop;
        pub use desktop::*;
    }
}
