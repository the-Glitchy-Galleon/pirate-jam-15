cfg_if::cfg_if! {
    if #[cfg(target_family = "wasm")] {
        mod wasm;
        pub use wasm::{create_app, run_app};
    } else {
        mod desktop;
        pub use desktop::{create_app, run_app};
    }
}
