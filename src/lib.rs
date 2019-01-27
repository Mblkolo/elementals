extern crate cfg_if;
extern crate nalgebra as na;
extern crate pyro;
extern crate rand;
extern crate serde_derive;
extern crate serde_json;
extern crate wasm_bindgen;

pub mod ecs;
pub mod facade;
pub mod game;
mod utils;

use cfg_if::cfg_if;
use wasm_bindgen::prelude::*;

cfg_if! {
    // When the `wee_alloc` feature is enabled, use `wee_alloc` as the global
    // allocator.
    if #[cfg(feature = "wee_alloc")] {
        extern crate wee_alloc;
        #[global_allocator]
        static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;
    }
}

#[wasm_bindgen]
extern "C" {
    fn alert(s: &str);
}

#[wasm_bindgen]
pub fn greet() -> game::Point {
    alert("Ничёси! Это работает!");

    game::Point { x: 0.5, y: -0.234 }
}
