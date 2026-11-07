// Autor: Felipe Bustos | Fecha: 2026-10-31
// Separa presentación, servicios y persistencia para la transición desde Tauri.
pub mod controller;
pub mod database;
pub mod error;
pub mod health_repository;
pub mod health_service;
pub mod product_repository;
pub mod product_service;

slint::include_modules!();
