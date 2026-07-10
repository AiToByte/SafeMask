// 允许库 crate 有未使用的公共 API 表面（在积极开发中）
// 随着更多前端功能集成，这些将逐步减少
#![allow(dead_code)]

pub mod api;
pub mod common;
pub mod core;
pub mod infra;

// 可以在这里做一些统一的初始化导出
pub use common::state::AppState;

