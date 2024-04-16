pub mod camera;
pub mod model;
pub mod pipeline;
pub mod resources;
pub mod state;
pub mod texture;
pub mod utils;
pub mod window;

use crate::window::run;

#[cfg(not(target_arch = "wasm32"))]
pub fn start(url: &str) {
    #[cfg(not(target_arch = "wasm32"))]
    if url.len() > 0 {
        pollster::block_on(run(url));
    }
}

#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::*;

#[cfg(target_arch = "wasm32")]
#[wasm_bindgen]
pub async fn start(url: String) {
    #[cfg(target_arch = "wasm32")]
    if url.len() > 0{
        run(&url).await;
    }
}


