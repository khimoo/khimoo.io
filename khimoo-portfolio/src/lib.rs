pub mod home;

// Only include article_processing for non-WASM targets
#[cfg(not(target_arch = "wasm32"))]
pub mod article_processing;

// Re-export commonly used types (only for non-WASM)
#[cfg(not(target_arch = "wasm32"))]
pub use article_processing::{
    ArticleMetadata, 
    ExtractedLink, 
    LinkType, 
    LinkExtractor,
    FrontMatterParser
};