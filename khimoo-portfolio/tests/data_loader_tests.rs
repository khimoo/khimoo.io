#![cfg(target_arch = "wasm32")]

use khimoo_portfolio::home::data_loader::{DataLoadError, DataLoader};

#[test]
fn test_data_loader_creation() {
    let loader = DataLoader::new();
    assert_eq!(loader.base_url, "/data");
}

#[test]
fn test_data_loader_with_custom_base_url() {
    let loader = DataLoader::with_base_url("/custom/path".to_string());
    assert_eq!(loader.base_url, "/custom/path");
}

#[test]
fn test_error_display() {
    let error = DataLoadError::NetworkError("Test error".to_string());
    assert_eq!(error.to_string(), "Network error: Test error");
} 