# Design Document

## Overview

インタラクティブなマインドマップ形式のポートフォリオサイトは、Yew（Rust WebAssembly）フレームワークとRapier2D物理エンジンを使用して構築されます。現在の実装を拡張し、Markdownベースの記事管理システムと、動的なノード表示システムを統合します。

### 技術スタック
- **フロントエンド**: Yew (Rust WebAssembly)
- **物理エンジン**: Rapier2D
- **ルーティング**: yew-router
- **Markdown処理**: pulldown-cmark
- **スタイリング**: インラインCSS（将来的にCSS-in-Rustライブラリの検討）
- **CI/CD**: GitHub Actions
- **記事管理**: Rust CLIツール（メタデータ抽出、リンク解析、関連性計算）
- **データ生成**: 静的JSON生成（ビルド時・ローカル開発時）
- **開発環境**: Nix（依存関係管理、再現可能なビルド環境）
- **ローカル開発**: Rust CLIツール + Nix shell（統一的な開発体験）

### 重要な開発環境の注意事項

**Nix環境でのコマンド実行**
- このプロジェクトはNix flakeを使用して開発環境を管理しています
- 全てのRustコマンド（cargo build、cargo test、cargo runなど）はNix環境内で実行する必要があります
- **重要**: `nix develop`を単独で実行すると、Kiro IDEでは終了判定が正しく行われません
- **推奨方法**: `nix develop --command [実行したいコマンド]`の形式でワンライナーとして実行してください

**コマンド実行例**:
```bash
# テスト実行
nix develop --command cargo test

# ビルド実行
nix develop --command cargo build

# 記事処理ツール実行
nix develop --command cargo run --bin process-articles

# 開発サーバー起動
nix develop --command trunk serve

# justコマンド使用
nix develop --command just dev
```

この方式により、Nix環境の依存関係を正しく利用しながら、Kiro IDEでの開発を円滑に進めることができます。

## Architecture

### 全体アーキテクチャ

```
┌─────────────────────────────────────────────────────────────┐
│                    GitHub Repository                        │
├─────────────────────────────────────────────────────────────┤
│  GitHub Actions Workflows                                  │
│  ┌─────────────────┐  ┌─────────────────┐  ┌──────────────┐ │
│  │ Article         │  │ Link Graph      │  │ Metadata     │ │
│  │ Processing      │  │ Generator       │  │ Validator    │ │
│  └─────────────────┘  └─────────────────┘  └──────────────┘ │
│           │                     │                   │       │
│  ┌─────────────────┐  ┌─────────────────┐  ┌──────────────┐ │
│  │ Markdown Files  │  │ Generated JSON  │  │ Build        │ │
│  │ (articles/*.md) │  │ (data/*.json)   │  │ Artifacts    │ │
│  └─────────────────┘  └─────────────────┘  └──────────────┘ │
├─────────────────────────────────────────────────────────────┤
│                    Browser (WebAssembly)                    │
├─────────────────────────────────────────────────────────────┤
│  Yew Application                                            │
│  ┌─────────────────┐  ┌─────────────────┐  ┌──────────────┐ │
│  │   Router        │  │  Home Component │  │ Article View │ │
│  │                 │  │                 │  │              │ │
│  └─────────────────┘  └─────────────────┘  └──────────────┘ │
│           │                     │                   │       │
│  ┌─────────────────┐  ┌─────────────────┐  ┌──────────────┐ │
│  │ Data Loader     │  │ Node Graph      │  │ Enhanced     │ │
│  │ (JSON Consumer) │  │ Container       │  │ Markdown     │ │
│  └─────────────────┘  └─────────────────┘  │ Renderer     │ │
│           │                     │          └──────────────┘ │
│  ┌─────────────────┐  ┌─────────────────┐  ┌──────────────┐ │
│  │ Static Data     │  │ Physics World   │  │ Interactive  │ │
│  │ (Pre-processed) │  │                 │  │ Navigation   │ │
│  └─────────────────┘  └─────────────────┘  └──────────────┘ │
├─────────────────────────────────────────────────────────────┤
│                    Static Assets                            │
│  ┌─────────────────┐  ┌─────────────────┐  ┌──────────────┐ │
│  │ Article Data    │  │ Link Graph      │  │ Images       │ │
│  │ (articles.json) │  │ (links.json)    │  │ (assets/*)   │ │
│  └─────────────────┘  └─────────────────┘  └──────────────┘ │
└─────────────────────────────────────────────────────────────┘
```

### データフロー

#### ビルド時（GitHub Actions / ローカル開発）
1. **記事検出**: Markdownファイルの変更を検出（push/PR時 or ローカル実行時）
2. **メタデータ抽出**: Front matterとMarkdown内容を解析
3. **リンク解析**: 記事間のリンク関係を抽出・検証
4. **JSON生成**: 処理済みデータを静的JSONファイルとして出力
5. **検証**: リンク切れ、メタデータ不整合をチェック・報告

#### ランタイム（ブラウザ）
1. **データ読み込み**: 事前生成されたJSONファイルを読み込み
2. **ノード生成**: home_display=trueの記事からノードを生成
3. **物理演算**: Rapier2Dエンジンがノードの位置を計算、リンク関係に基づく接続力を適用
4. **レンダリング**: Yewコンポーネントが物理演算結果を基にUIを描画、接続線をアニメーション表示
5. **インタラクティブナビゲーション**: ノードクリック時に関連記事をハイライト、記事内リンクで動的遷移
6. **デバッグUI**: デバッグモード時にノード間の結合力調整UIを表示

#### ローカル開発ワークフロー
1. **記事作成・編集**: `articles/`ディレクトリでMarkdownファイルを編集
2. **ローカル処理実行**: `npm run process-articles`または`cargo run --bin process-articles`
3. **リアルタイム更新**: ファイル監視による自動再処理（オプション）
4. **開発サーバー起動**: 生成されたJSONを使用してYewアプリケーションを実行

## Nix Development Environment

### 1. Nix Flake Configuration

#### flake.nix
```nix
{
  description = "Interactive Mindmap Portfolio";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    rust-overlay.url = "github:oxalica/rust-overlay";
    flake-utils.url = "github:numtide/flake-utils";
  };

  outputs = { self, nixpkgs, rust-overlay, flake-utils }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        overlays = [ (import rust-overlay) ];
        pkgs = import nixpkgs {
          inherit system overlays;
        };
        
        rustToolchain = pkgs.rust-bin.stable.latest.default.override {
          extensions = [ "rust-src" "rust-analyzer" ];
          targets = [ "wasm32-unknown-unknown" ];
        };
      in
      {
        devShells.default = pkgs.mkShell {
          buildInputs = with pkgs; [
            # Rust toolchain
            rustToolchain
            
            # WebAssembly tools
            wasm-pack
            trunk
            
            # Development tools
            watchexec
            just
            
            # System dependencies
            pkg-config
            openssl
          ];
          
          shellHook = ''
            echo "🦀 Rust WebAssembly development environment"
            echo "📦 Available commands:"
            echo "  just dev      - Start development server with file watching"
            echo "  just build    - Build for production"
            echo "  just process  - Process articles and generate data"
            echo "  just validate - Validate links and content"
            echo "  just clean    - Clean generated files"
          '';
        };
        
        packages.default = pkgs.rustPlatform.buildRustPackage {
          pname = "khimoo-portfolio";
          version = "0.1.0";
          src = ./.;
          cargoLock.lockFile = ./Cargo.lock;
          
          buildInputs = with pkgs; [ pkg-config openssl ];
          
          # WebAssembly build
          buildPhase = ''
            cargo build --release
            wasm-pack build --target web --out-dir pkg
          '';
        };
      });
}
```

### 2. Rust CLI Tools

#### Cargo.toml
```toml
[package]
name = "khimoo-portfolio"
version = "0.1.0"
edition = "2021"

# Main application
[[bin]]
name = "khimoo-portfolio"
path = "src/main.rs"

# Article processing tools
[[bin]]
name = "process-articles"
path = "src/bin/process_articles.rs"

[[bin]]
name = "validate-links"
path = "src/bin/validate_links.rs"

[[bin]]
name = "generate-link-graph"
path = "src/bin/generate_link_graph.rs"

[[bin]]
name = "dev-server"
path = "src/bin/dev_server.rs"

[dependencies]
# Web framework
yew = { version = "0.21", features = ["csr"] }
yew-router = "0.18"
web-sys = { version = "0.3", features = ["HtmlElement", "HtmlDivElement", "Element", "DomRect"] }
yew-hooks = "0.3"

# Physics
rapier2d = { version = "0.26", features = ["simd-stable"] }

# Markdown processing
pulldown-cmark = "0.10"

# CLI tools dependencies
clap = { version = "4.0", features = ["derive"] }
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
serde_yaml = "0.9"
walkdir = "2.3"
regex = "1.7"
notify = "6.0"
tokio = { version = "1.0", features = ["full"] }
anyhow = "1.0"
chrono = { version = "0.4", features = ["serde"] }

[dev-dependencies]
wasm-bindgen-test = "0.3"
```

#### justfile (Task Runner)
```just
# Default recipe
default:
    @just --list

# Start development environment
dev:
    @echo "🚀 Starting development environment..."
    @just process-articles
    @watchexec -w articles -e md -- just process-articles &
    @trunk serve

# Process all articles
process-articles:
    @echo "📝 Processing articles..."
    @cargo run --bin process-articles

# Validate links
validate-links:
    @echo "🔗 Validating links..."
    @cargo run --bin validate-links

# Generate link graph
generate-link-graph:
    @echo "🕸️  Generating link graph..."
    @cargo run --bin generate-link-graph

# Build all data
build-data: process-articles validate-links generate-link-graph
    @echo "✅ All data processed successfully"

# Build for production
build: build-data
    @echo "🏗️  Building for production..."
    @trunk build --release

# Clean generated files
clean:
    @echo "🧹 Cleaning up..."
    @rm -rf dist data/*.json target pkg

# Run tests
test:
    @echo "🧪 Running tests..."
    @cargo test
    @wasm-pack test --headless --firefox

# Watch articles and rebuild
watch:
    @echo "👀 Watching articles for changes..."
    @watchexec -w articles -e md -- just build-data

# Development server with hot reload
serve: build-data
    @echo "🌐 Starting development server..."
    @trunk serve --open
```

#### Article Processing CLI
```rust
// src/bin/process_articles.rs
use clap::Parser;
use anyhow::{Context, Result};
use std::path::PathBuf;
use serde::{Deserialize, Serialize};
use walkdir::WalkDir;
use regex::Regex;
use std::collections::HashMap;
use chrono::{DateTime, Utc};

#[derive(Parser)]
#[command(name = "process-articles")]
#[command(about = "Process articles and generate static data")]
struct Args {
    /// Articles directory path
    #[arg(short, long, default_value = "articles")]
    articles_dir: PathBuf,
    
    /// Output directory for generated data
    #[arg(short, long, default_value = "data")]
    output_dir: PathBuf,
    
    /// Enable verbose logging
    #[arg(short, long)]
    verbose: bool,
}

#[tokio::main]
async fn main() -> Result<()> {
    let args = Args::parse();
    
    let processor = ArticleProcessor::new(args.articles_dir, args.output_dir, args.verbose);
    processor.process_all_articles().await
}

pub struct ArticleProcessor {
    articles_dir: PathBuf,
    output_dir: PathBuf,
    verbose: bool,
}

impl ArticleProcessor {
    pub fn new(articles_dir: PathBuf, output_dir: PathBuf, verbose: bool) -> Self {
        Self { articles_dir, output_dir, verbose }
    }

    pub async fn process_all_articles(&self) -> Result<()> {
        if self.verbose {
            println!("🔄 Processing articles from {:?}", self.articles_dir);
        }
        
        // Create output directory
        std::fs::create_dir_all(&self.output_dir)
            .context("Failed to create output directory")?;
        
        // Load and parse all articles
        let articles = self.load_and_parse_articles()
            .context("Failed to load articles")?;
        
        if self.verbose {
            println!("📚 Found {} articles", articles.len());
        }
        
        // Build link graph
        let link_graph = self.build_link_graph(&articles)
            .context("Failed to build link graph")?;
        
        // Validate content
        let validation_report = self.validate_content(&articles, &link_graph)
            .context("Failed to validate content")?;
        
        // Write output files
        self.write_articles_data(&articles)
            .context("Failed to write articles data")?;
        self.write_link_graph_data(&link_graph)
            .context("Failed to write link graph data")?;
        self.write_validation_report(&validation_report)
            .context("Failed to write validation report")?;
        
        println!("✅ Successfully processed {} articles", articles.len());
        
        if !validation_report.errors.is_empty() {
            println!("⚠️  Found {} validation errors", validation_report.errors.len());
            for error in &validation_report.errors {
                println!("   - {}: {} -> {}", error.error_type, error.source, error.target);
            }
        }
        
        Ok(())
    }

    fn load_and_parse_articles(&self) -> Result<Vec<ProcessedArticle>> {
        let mut articles = Vec::new();
        
        for entry in WalkDir::new(&self.articles_dir)
            .into_iter()
            .filter_map(|e| e.ok())
            .filter(|e| e.path().extension().map_or(false, |ext| ext == "md"))
        {
            let content = std::fs::read_to_string(entry.path())
                .with_context(|| format!("Failed to read file: {:?}", entry.path()))?;
            
            let article = self.parse_article(entry.path(), &content)
                .with_context(|| format!("Failed to parse article: {:?}", entry.path()))?;
            
            articles.push(article);
        }
        
        Ok(articles)
    }

    fn parse_article(&self, file_path: &std::path::Path, content: &str) -> Result<ProcessedArticle> {
        // Parse front matter
        let (metadata, markdown_content) = self.parse_front_matter(content)?;
        
        // Extract links
        let outbound_links = self.extract_links(&markdown_content)?;
        
        // Extract tags from content
        let extracted_tags = self.extract_tags(&markdown_content);
        
        // Generate slug from file path
        let slug = self.generate_slug(file_path);
        
        Ok(ProcessedArticle {
            slug,
            title: metadata.title.clone(),
            content: markdown_content,
            metadata,
            file_path: file_path.to_string_lossy().to_string(),
            outbound_links,
            inbound_count: 0, // Will be calculated later
            extracted_tags,
            processed_at: Utc::now().to_rfc3339(),
        })
    }

    fn parse_front_matter(&self, content: &str) -> Result<(ProcessedMetadata, String)> {
        if !content.starts_with("---\n") {
            return Ok((ProcessedMetadata::default(), content.to_string()));
        }
        
        let end_marker = content[4..].find("\n---\n")
            .ok_or_else(|| anyhow::anyhow!("Invalid front matter: missing end marker"))?;
        
        let yaml_content = &content[4..end_marker + 4];
        let markdown_content = &content[end_marker + 8..];
        
        let metadata: ProcessedMetadata = serde_yaml::from_str(yaml_content)
            .context("Failed to parse YAML front matter")?;
        
        Ok((metadata, markdown_content.to_string()))
    }

    fn extract_links(&self, content: &str) -> Result<Vec<ProcessedLink>> {
        let mut links = Vec::new();
        
        // Extract [[wiki-style]] links
        let wiki_regex = Regex::new(r"\[\[([^\]]+)\]\]")?;
        for cap in wiki_regex.captures_iter(content) {
            let target = cap.get(1).unwrap().as_str();
            let position = cap.get(0).unwrap().start();
            
            links.push(ProcessedLink {
                target_slug: self.generate_slug_from_title(target),
                link_type: LinkType::WikiLink,
                context: self.get_context(content, position, 50),
                position,
            });
        }
        
        // Extract [text](slug) links
        let markdown_regex = Regex::new(r"\[([^\]]+)\]\(([^)]+)\)")?;
        for cap in markdown_regex.captures_iter(content) {
            let target = cap.get(2).unwrap().as_str();
            let position = cap.get(0).unwrap().start();
            
            // Only process internal links (not starting with http)
            if !target.starts_with("http") {
                links.push(ProcessedLink {
                    target_slug: target.to_string(),
                    link_type: LinkType::MarkdownLink,
                    context: self.get_context(content, position, 50),
                    position,
                });
            }
        }
        
        Ok(links)
    }

    fn extract_tags(&self, content: &str) -> Vec<String> {
        // タグ抽出は記録のみ行い、関連性計算には使用しない
        let tag_regex = Regex::new(r"#([a-zA-Z0-9_-]+)").unwrap();
        tag_regex
            .captures_iter(content)
            .map(|cap| cap.get(1).unwrap().as_str().to_string())
            .collect::<std::collections::HashSet<_>>()
            .into_iter()
            .collect()
    }

    fn generate_slug(&self, file_path: &std::path::Path) -> String {
        file_path
            .file_stem()
            .unwrap()
            .to_string_lossy()
            .to_string()
            .to_lowercase()
            .replace(' ', "-")
    }

    fn generate_slug_from_title(&self, title: &str) -> String {
        title
            .to_lowercase()
            .replace(' ', "-")
            .chars()
            .filter(|c| c.is_alphanumeric() || *c == '-')
            .collect()
    }

    fn get_context(&self, content: &str, position: usize, length: usize) -> String {
        let start = position.saturating_sub(length / 2);
        let end = std::cmp::min(position + length / 2, content.len());
        content[start..end].to_string()
    }

    fn build_link_graph(&self, articles: &[ProcessedArticle]) -> Result<LinkGraphData> {
        let mut graph = HashMap::new();
        let article_slugs: std::collections::HashSet<_> = 
            articles.iter().map(|a| &a.slug).collect();
        
        for article in articles {
            let mut connections = Vec::new();
            
            // Process outbound links (直接リンクのみ)
            for link in &article.outbound_links {
                if article_slugs.contains(&link.target_slug) {
                    connections.push(GraphConnection {
                        target: link.target_slug.clone(),
                        connection_type: ConnectionType::DirectLink,
                        bidirectional: false,
                    });
                }
            }
            
            graph.insert(article.slug.clone(), GraphNode {
                connections,
                inbound_count: 0, // Will be calculated in next pass
            });
        }
        
        // Calculate inbound counts
        for article in articles {
            for link in &article.outbound_links {
                if let Some(target_node) = graph.get_mut(&link.target_slug) {
                    target_node.inbound_count += 1;
                }
            }
        }
        
        Ok(LinkGraphData {
            graph,
            generated_at: Utc::now().to_rfc3339(),
            total_connections: graph.values()
                .map(|node| node.connections.len())
                .sum(),
        })
    }

    // リンク強度の概念は削除し、デバッグモードでの調整に変更

    fn validate_content(
        &self,
        articles: &[ProcessedArticle],
        _link_graph: &LinkGraphData,
    ) -> Result<ValidationReport> {
        let mut errors = Vec::new();
        let mut warnings = Vec::new();
        
        let existing_slugs: std::collections::HashSet<_> = 
            articles.iter().map(|a| &a.slug).collect();
        
        // Validate internal links
        for article in articles {
            for link in &article.outbound_links {
                if !existing_slugs.contains(&link.target_slug) {
                    errors.push(ValidationError {
                        error_type: "broken_link".to_string(),
                        source: article.slug.clone(),
                        target: link.target_slug.clone(),
                        context: Some(link.context.clone()),
                    });
                }
            }
            
            // Validate metadata references
            for related_slug in &article.metadata.related_articles {
                if !existing_slugs.contains(related_slug) {
                    warnings.push(ValidationWarning {
                        warning_type: "invalid_related_article".to_string(),
                        source: article.slug.clone(),
                        target: related_slug.clone(),
                        context: None,
                    });
                }
            }
        }
        
        Ok(ValidationReport {
            validation_date: Utc::now().to_rfc3339(),
            total_articles: articles.len(),
            errors,
            warnings,
            summary: ValidationSummary {
                broken_links: errors.len(),
                invalid_references: warnings.len(),
            },
        })
    }

    fn write_articles_data(&self, articles: &[ProcessedArticle]) -> Result<()> {
        let articles_data = ArticlesData {
            articles: articles.to_vec(),
            generated_at: Utc::now().to_rfc3339(),
            total_count: articles.len(),
            home_articles: articles
                .iter()
                .filter(|a| a.metadata.home_display)
                .map(|a| a.slug.clone())
                .collect(),
        };
        
        let output_path = self.output_dir.join("articles.json");
        let json = serde_json::to_string_pretty(&articles_data)?;
        std::fs::write(output_path, json)?;
        
        Ok(())
    }

    fn write_link_graph_data(&self, link_graph: &LinkGraphData) -> Result<()> {
        let output_path = self.output_dir.join("link-graph.json");
        let json = serde_json::to_string_pretty(link_graph)?;
        std::fs::write(output_path, json)?;
        
        Ok(())
    }

    fn write_validation_report(&self, report: &ValidationReport) -> Result<()> {
        let output_path = self.output_dir.join("validation-report.json");
        let json = serde_json::to_string_pretty(report)?;
        std::fs::write(output_path, json)?;
        
        Ok(())
    }
}

// Data structures
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProcessedArticle {
    pub slug: String,
    pub title: String,
    pub content: String,
    pub metadata: ProcessedMetadata,
    pub file_path: String,
    pub outbound_links: Vec<ProcessedLink>,
    pub inbound_count: usize,
    pub extracted_tags: Vec<String>,
    pub processed_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProcessedMetadata {
    pub title: String,
    pub home_display: bool,
    pub category: Option<String>,
    pub importance: Option<u8>,
    pub related_articles: Vec<String>,
    pub tags: Vec<String>,
    pub created_at: Option<String>,
    pub updated_at: Option<String>,
}

impl Default for ProcessedMetadata {
    fn default() -> Self {
        Self {
            title: "Untitled".to_string(),
            home_display: false,
            category: None,
            importance: Some(3),
            related_articles: Vec::new(),
            tags: Vec::new(),
            created_at: None,
            updated_at: None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProcessedLink {
    pub target_slug: String,
    pub link_type: LinkType,
    pub context: String,
    pub position: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum LinkType {
    WikiLink,
    MarkdownLink,
    TagReference,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArticlesData {
    pub articles: Vec<ProcessedArticle>,
    pub generated_at: String,
    pub total_count: usize,
    pub home_articles: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LinkGraphData {
    pub graph: HashMap<String, GraphNode>,
    pub generated_at: String,
    pub total_connections: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GraphNode {
    pub connections: Vec<GraphConnection>,
    pub inbound_count: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GraphConnection {
    pub target: String,
    pub connection_type: ConnectionType,
    pub bidirectional: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ConnectionType {
    DirectLink,
    Bidirectional,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationReport {
    pub validation_date: String,
    pub total_articles: usize,
    pub errors: Vec<ValidationError>,
    pub warnings: Vec<ValidationWarning>,
    pub summary: ValidationSummary,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationError {
    pub error_type: String,
    pub source: String,
    pub target: String,
    pub context: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationWarning {
    pub warning_type: String,
    pub source: String,
    pub target: String,
    pub context: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationSummary {
    pub broken_links: usize,
    pub invalid_references: usize,
}
```

### 2. Development Server Integration

#### trunk.toml Configuration
```toml
[build]
target = "index.html"
dist = "dist"

[watch]
watch = ["src", "data"]
ignore = ["target"]

[serve]
address = "127.0.0.1"
port = 8080
open = false

[[hooks]]
stage = "pre_build"
command = "npm"
command_arguments = ["run", "build-data"]
```

#### Development Makefile
```makefile
.PHONY: dev build clean process-articles watch-articles

# ローカル開発環境の起動
dev:
	@echo "🚀 Starting development environment..."
	@npm run dev

# 記事データの処理
process-articles:
	@echo "📝 Processing articles..."
	@npm run build-data

# 記事の監視モード
watch-articles:
	@echo "👀 Watching articles for changes..."
	@npm run watch-articles

# プロダクションビルド
build: process-articles
	@echo "🏗️  Building for production..."
	@trunk build --release

# クリーンアップ
clean:
	@echo "🧹 Cleaning up..."
	@rm -rf dist data/*.json target

# GitHub Actionsと同じ処理をローカルで実行
ci-local: process-articles
	@echo "🔄 Running CI pipeline locally..."
	@npm run validate-links
	@cargo test
	@trunk build --release
```

## GitHub Workflows

### 1. Article Processing Workflow

#### Trigger Conditions
- Push to main branch with changes in `articles/` directory
- Pull request with article modifications
- Manual workflow dispatch for full rebuild

#### Workflow Steps

```yaml
name: Process Articles
on:
  push:
    paths: ['articles/**/*.md']
  pull_request:
    paths: ['articles/**/*.md']
  workflow_dispatch:

jobs:
  process-articles:
    runs-on: ubuntu-latest
    steps:
      - name: Checkout repository
        uses: actions/checkout@v4
        
      - name: Install Nix
        uses: cachix/install-nix-action@v22
        with:
          github_access_token: ${{ secrets.GITHUB_TOKEN }}
          
      - name: Setup development environment
        run: nix develop --command echo "Environment ready"
        
      - name: Process articles
        run: nix develop --command just build-data
        
      - name: Run tests
        run: nix develop --command just test
        
      - name: Commit generated data
        uses: stefanzweifel/git-auto-commit-action@v5
        with:
          commit_message: 'Auto-update article data and link graph'
          file_pattern: 'data/*.json'
```

#### Build and Deploy Workflow
```yaml
name: Build and Deploy
on:
  push:
    branches: [main]
  workflow_run:
    workflows: ["Process Articles"]
    types: [completed]

jobs:
  build-and-deploy:
    runs-on: ubuntu-latest
    steps:
      - name: Checkout repository
        uses: actions/checkout@v4
        
      - name: Install Nix
        uses: cachix/install-nix-action@v22
        with:
          github_access_token: ${{ secrets.GITHUB_TOKEN }}
          
      - name: Build application
        run: nix develop --command just build
        
      - name: Deploy to GitHub Pages
        uses: peaceiris/actions-gh-pages@v3
        with:
          github_token: ${{ secrets.GITHUB_TOKEN }}
          publish_dir: ./dist
```

### 2. Article Processing Scripts

#### process-articles.js
```javascript
const fs = require('fs');
const path = require('path');
const matter = require('gray-matter');
const MarkdownIt = require('markdown-it');

class ArticleProcessor {
  constructor() {
    this.articlesDir = 'articles';
    this.outputDir = 'data';
    this.articles = new Map();
    this.linkGraph = new Map();
  }

  async processAllArticles() {
    // 全記事を読み込み・解析
    const files = this.getMarkdownFiles();
    
    for (const file of files) {
      const article = await this.processArticle(file);
      this.articles.set(article.slug, article);
    }
    
    // リンクグラフを構築
    this.buildLinkGraph();
    
    // JSONファイルを出力
    this.writeArticlesData();
    this.writeLinkGraphData();
  }

  processArticle(filePath) {
    const content = fs.readFileSync(filePath, 'utf8');
    const { data: frontMatter, content: markdown } = matter(content);
    
    const slug = this.generateSlug(filePath);
    const links = this.extractLinks(markdown);
    const tags = this.extractTags(markdown);
    
    return {
      slug,
      title: frontMatter.title || path.basename(filePath, '.md'),
      content: markdown,
      metadata: {
        home_display: frontMatter.home_display || false,
        category: frontMatter.category,
        importance: frontMatter.importance || 3,
        related_articles: frontMatter.related_articles || [],
        tags: [...(frontMatter.tags || []), ...tags],
        created_at: frontMatter.created_at,
        updated_at: frontMatter.updated_at || new Date().toISOString()
      },
      outbound_links: links,
      file_path: filePath
    };
  }

  extractLinks(markdown) {
    const links = [];
    
    // [[記事名]]形式のリンクを抽出
    const wikiLinkRegex = /\[\[([^\]]+)\]\]/g;
    let match;
    while ((match = wikiLinkRegex.exec(markdown)) !== null) {
      links.push({
        target_slug: this.generateSlug(match[1]),
        link_type: 'WikiLink',
        context: this.getContext(markdown, match.index),
        position: match.index
      });
    }
    
    // [テキスト](slug)形式のリンクを抽出
    const markdownLinkRegex = /\[([^\]]+)\]\(([^)]+)\)/g;
    while ((match = markdownLinkRegex.exec(markdown)) !== null) {
      if (!match[2].startsWith('http')) { // 内部リンクのみ
        links.push({
          target_slug: match[2],
          link_type: 'MarkdownLink',
          context: this.getContext(markdown, match.index),
          position: match.index
        });
      }
    }
    
    return links;
  }

  extractTags(markdown) {
    const tagRegex = /#([a-zA-Z0-9_-]+)/g;
    const tags = [];
    let match;
    
    while ((match = tagRegex.exec(markdown)) !== null) {
      tags.push(match[1]);
    }
    
    return [...new Set(tags)]; // 重複除去
  }

  buildLinkGraph() {
    const graph = new Map();
    
    for (const [slug, article] of this.articles) {
      if (!graph.has(slug)) {
        graph.set(slug, { connections: [], inbound_count: 0 });
      }
      
      // アウトバウンドリンクを処理
      for (const link of article.outbound_links) {
        if (this.articles.has(link.target_slug)) {
          graph.get(slug).connections.push({
            target: link.target_slug,
            type: link.link_type,
            strength: this.calculateLinkStrength(article, link)
          });
          
          // インバウンドカウントを更新
          if (!graph.has(link.target_slug)) {
            graph.set(link.target_slug, { connections: [], inbound_count: 0 });
          }
          graph.get(link.target_slug).inbound_count++;
        }
      }
      
      // タグベースの関連性を計算
      this.addTagBasedConnections(graph, slug, article);
    }
    
    this.linkGraph = graph;
  }

  calculateLinkStrength(fromArticle, link) {
    let strength = 0.5; // ベース強度
    
    // リンクタイプによる調整
    if (link.link_type === 'WikiLink') strength += 0.2;
    if (link.link_type === 'MarkdownLink') strength += 0.1;
    
    // 記事の重要度による調整
    strength += (fromArticle.metadata.importance - 3) * 0.1;
    
    return Math.min(1.0, Math.max(0.1, strength));
  }

  addTagBasedConnections(graph, slug, article) {
    for (const [otherSlug, otherArticle] of this.articles) {
      if (slug === otherSlug) continue;
      
      const commonTags = article.metadata.tags.filter(tag => 
        otherArticle.metadata.tags.includes(tag)
      );
      
      if (commonTags.length > 0) {
        const strength = Math.min(0.8, commonTags.length * 0.2);
        
        graph.get(slug).connections.push({
          target: otherSlug,
          type: 'TagBased',
          strength,
          common_tags: commonTags
        });
      }
    }
  }

  writeArticlesData() {
    const articlesData = {
      articles: Array.from(this.articles.values()),
      generated_at: new Date().toISOString(),
      total_count: this.articles.size,
      home_articles: Array.from(this.articles.values())
        .filter(a => a.metadata.home_display)
        .map(a => a.slug)
    };
    
    fs.writeFileSync(
      path.join(this.outputDir, 'articles.json'),
      JSON.stringify(articlesData, null, 2)
    );
  }

  writeLinkGraphData() {
    const linkGraphData = {
      graph: Object.fromEntries(this.linkGraph),
      generated_at: new Date().toISOString(),
      total_connections: Array.from(this.linkGraph.values())
        .reduce((sum, node) => sum + node.connections.length, 0)
    };
    
    fs.writeFileSync(
      path.join(this.outputDir, 'link-graph.json'),
      JSON.stringify(linkGraphData, null, 2)
    );
  }
}

// 実行
const processor = new ArticleProcessor();
processor.processAllArticles().catch(console.error);
```

#### validate-links.js
```javascript
const fs = require('fs');
const path = require('path');

class LinkValidator {
  constructor() {
    this.articlesData = JSON.parse(fs.readFileSync('data/articles.json', 'utf8'));
    this.errors = [];
    this.warnings = [];
  }

  validate() {
    this.validateInternalLinks();
    this.validateMetadataReferences();
    this.generateReport();
  }

  validateInternalLinks() {
    const existingSlugs = new Set(this.articlesData.articles.map(a => a.slug));
    
    for (const article of this.articlesData.articles) {
      for (const link of article.outbound_links) {
        if (!existingSlugs.has(link.target_slug)) {
          this.errors.push({
            type: 'broken_link',
            source: article.slug,
            target: link.target_slug,
            context: link.context
          });
        }
      }
    }
  }

  validateMetadataReferences() {
    const existingSlugs = new Set(this.articlesData.articles.map(a => a.slug));
    
    for (const article of this.articlesData.articles) {
      for (const relatedSlug of article.metadata.related_articles) {
        if (!existingSlugs.has(relatedSlug)) {
          this.warnings.push({
            type: 'invalid_related_article',
            source: article.slug,
            target: relatedSlug
          });
        }
      }
    }
  }

  generateReport() {
    const report = {
      validation_date: new Date().toISOString(),
      total_articles: this.articlesData.articles.length,
      errors: this.errors,
      warnings: this.warnings,
      summary: {
        broken_links: this.errors.filter(e => e.type === 'broken_link').length,
        invalid_references: this.warnings.filter(w => w.type === 'invalid_related_article').length
      }
    };
    
    fs.writeFileSync('data/validation-report.json', JSON.stringify(report, null, 2));
    
    // GitHub Actionsでの表示用
    if (this.errors.length > 0) {
      console.error('❌ Validation errors found:');
      this.errors.forEach(error => {
        console.error(`  - ${error.type}: ${error.source} -> ${error.target}`);
      });
      process.exit(1);
    }
    
    if (this.warnings.length > 0) {
      console.warn('⚠️  Validation warnings:');
      this.warnings.forEach(warning => {
        console.warn(`  - ${warning.type}: ${warning.source} -> ${warning.target}`);
      });
    }
    
    console.log('✅ Link validation completed successfully');
  }
}

const validator = new LinkValidator();
validator.validate();
```

### 3. Deployment Integration

#### Build Workflow
```yaml
name: Build and Deploy
on:
  push:
    branches: [main]
  workflow_run:
    workflows: ["Process Articles"]
    types: [completed]

jobs:
  build:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      
      - name: Setup Rust
        uses: actions-rs/toolchain@v1
        with:
          toolchain: stable
          target: wasm32-unknown-unknown
          
      - name: Install wasm-pack
        run: curl https://rustwasm.github.io/wasm-pack/installer/init.sh -sSf | sh
        
      - name: Build WebAssembly
        run: wasm-pack build --target web --out-dir pkg
        
      - name: Deploy to GitHub Pages
        uses: peaceiris/actions-gh-pages@v3
        with:
          github_token: ${{ secrets.GITHUB_TOKEN }}
          publish_dir: ./dist
```

## Components and Interfaces

### 1. Article Management System

#### DataLoader (GitHub Workflow Integration)
```rust
pub struct DataLoader {
    articles_data: ArticlesData,               // 事前生成されたJSONデータ
    link_graph_data: LinkGraphData,            // 事前生成されたリンクグラフ
    validation_report: ValidationReport,       // リンク検証レポート
}

impl DataLoader {
    pub async fn new() -> Result<Self, LoadError>;
    pub async fn load_articles_data() -> Result<ArticlesData, LoadError>;
    pub async fn load_link_graph_data() -> Result<LinkGraphData, LoadError>;
    pub async fn load_validation_report() -> Result<ValidationReport, LoadError>;
    pub fn get_home_articles(&self) -> Vec<&ProcessedArticle>;
    pub fn get_article_by_slug(&self, slug: &str) -> Option<&ProcessedArticle>;
    pub fn get_related_articles(&self, slug: &str) -> Vec<&ProcessedArticle>;
    pub fn get_connection_strength(&self, from: &str, to: &str) -> f32;
}

#[derive(Deserialize)]
pub struct ArticlesData {
    pub articles: Vec<ProcessedArticle>,
    pub generated_at: String,
    pub total_count: usize,
    pub home_articles: Vec<String>,            // home_display=trueの記事slugs
}

#[derive(Deserialize)]
pub struct LinkGraphData {
    pub graph: HashMap<String, GraphNode>,     // slug -> connections
    pub generated_at: String,
    pub total_connections: usize,
}

#[derive(Deserialize)]
pub struct ValidationReport {
    pub validation_date: String,
    pub total_articles: usize,
    pub errors: Vec<ValidationError>,
    pub warnings: Vec<ValidationWarning>,
    pub summary: ValidationSummary,
}

#### ArticleManager (Runtime)
```rust
pub struct ArticleManager {
    data_loader: DataLoader,
    articles: HashMap<String, ProcessedArticle>, // slug -> Article
    home_articles: Vec<String>,                // home_display=trueの記事のslug
    category_index: HashMap<String, Vec<String>>, // category -> slugs
    tag_index: HashMap<String, Vec<String>>,   // tag -> slugs
    link_graph: LinkGraph,                     // 記事間のリンク関係
}

impl ArticleManager {
    pub async fn new() -> Result<Self, LoadError>;
    pub async fn initialize_from_data_loader() -> Result<Self, LoadError>;
    pub fn get_home_articles(&self) -> Vec<&ProcessedArticle>;
    pub fn get_article_by_slug(&self, slug: &str) -> Option<&ProcessedArticle>;
    pub fn get_related_articles(&self, slug: &str) -> Vec<&ProcessedArticle>;
    pub fn get_relationship_strength(&self, from: &str, to: &str) -> f32; // 事前計算済み
    pub fn get_validation_status(&self) -> &ValidationReport;
}

// GitHub Workflowで事前処理された記事データ
#[derive(Deserialize, Clone)]
pub struct ProcessedArticle {
    pub slug: String,                          // ファイル名から生成
    pub title: String,                         // メタデータのtitleまたはファイル名
    pub content: String,                       // Markdown本文
    pub metadata: ProcessedMetadata,           // 事前処理済みメタデータ
    pub file_path: String,                     // 元ファイルパス
    pub outbound_links: Vec<ProcessedLink>,    // この記事から他記事へのリンク
    pub inbound_count: usize,                  // この記事を参照している記事数
    pub extracted_tags: Vec<String>,           // 記事本文から抽出されたタグ
    pub processed_at: String,                  // 処理日時
}

// ランタイムでの軽量な記事表現（必要に応じて元データから復元）
pub struct Article {
    pub slug: String,
    pub title: String,
    pub content: Option<String>,               // 遅延読み込み対応
    pub metadata: ProcessedMetadata,
    pub connections: Vec<Connection>,          // 事前計算済み接続情報
}

impl Article {
    pub fn from_processed(processed: &ProcessedArticle, connections: Vec<Connection>) -> Sel

#[derive(Debug, Clone)]
pub struct ArticleLink {
    pub target_slug: String,                   // リンク先のslug
    pub link_type: LinkType,                   // リンクの種類
    pub context: String,                       // リンク周辺のテキスト
    pub position: usize,                       // 記事内での位置
}

#[derive(Debug, Clone, PartialEq)]
pub enum LinkType {
    WikiLink,      // [[記事名]]形式
    MarkdownLink,  // [テキスト](slug)形式
    TagReference,  // #tag形式
}
```

#### ArticleLoader
```rust
pub struct ArticleLoader {
    base_path: String,
}

impl ArticleLoader {
    pub fn new(base_path: &str) -> Self;
    pub fn load_all_articles(&self) -> Result<Vec<Article>, LoadError>;
    pub fn load_article_from_file(&self, file_path: &str) -> Result<Article, LoadError>;
    pub fn watch_for_changes(&self) -> impl Stream<Item = ArticleEvent>; // 将来の拡張用
}

pub enum ArticleEvent {
    Created(String),
    Updated(String),
    Deleted(String),
}
```

### 2. Enhanced Node System

#### NodeType
```rust
pub enum NodeType {
    AuthorProfile {
        image_url: String,
        name: String,
    },
    Article {
        slug: String,
        title: String,
        category: Option<String>,
        importance: u8,
    },
}

pub struct EnhancedNode {
    id: NodeId,
    node_type: NodeType,
    position: Position,
    radius: i32,
    connections: Vec<NodeId>,
}
```

#### NodeFactory
```rust
pub struct NodeFactory;

impl NodeFactory {
    pub fn create_author_node(metadata: &AuthorMetadata) -> EnhancedNode;
    pub fn create_article_node(article: &Article) -> EnhancedNode;
    pub fn calculate_node_size(importance: u8, base_size: i32) -> i32;
    pub fn get_category_color(category: &str) -> String;
}
```

### 3. Enhanced Physics System

#### PhysicsConfiguration
```rust
pub struct PhysicsConfiguration {
    pub force_settings: ForceSettings,
    pub author_node_settings: AuthorNodeSettings,
    pub layout_constraints: LayoutConstraints,
}

pub struct AuthorNodeSettings {
    pub fixed_position: bool,
    pub center_weight: f32, // 中心への引力の重み
}

pub struct LayoutConstraints {
    pub max_distance_from_center: f32,
    pub min_node_distance: f32,
    pub category_clustering: bool,
}
```

### 4. Responsive Layout System

#### ViewportManager
```rust
pub struct ViewportManager {
    pub viewport: Viewport,
    pub device_type: DeviceType,
    pub touch_handler: TouchHandler,
}

pub enum DeviceType {
    Desktop,
    Tablet,
    Mobile,
}

pub struct TouchHandler {
    pub zoom_enabled: bool,
    pub pan_enabled: bool,
    pub node_drag_enabled: bool,
}
```

### 5. Visual Enhancement System

#### NodeRenderer
```rust
pub struct NodeRenderer;

impl NodeRenderer {
    pub fn render_author_node(node: &EnhancedNode) -> Html;
    pub fn render_article_node(node: &EnhancedNode) -> Html;
    pub fn create_tooltip(article: &Article) -> Html;
    pub fn apply_category_styling(category: &str) -> String;
}
```

#### ConnectionRenderer
```rust
pub struct ConnectionRenderer;

impl ConnectionRenderer {
    pub fn render_connections(nodes: &[EnhancedNode]) -> Html;
    pub fn create_animated_line(from: Position, to: Position) -> Html;
    pub fn apply_connection_styling(connection_type: ConnectionType) -> String;
}
```

## Data Models

### Article Data Model

#### Front Matter形式
各Markdownファイルの先頭にYAML形式でメタデータを記述：

```markdown
---
title: "Rustでの非同期プログラミング"
home_display: true
category: "programming"
importance: 4
related_articles: ["async-patterns", "tokio-basics"]
tags: ["rust", "async", "programming"]
created_at: "2024-01-15"
updated_at: "2024-01-20"
---

# 記事の内容
実際のMarkdownコンテンツがここに続きます...
```

#### Rustデータ構造

```rust
pub struct ArticleMetadata {
    pub title: String,
    pub home_display: bool,                    // ホーム画面に表示するか
    pub category: Option<String>,              // カテゴリ（自由入力）
    pub importance: Option<u8>,                // 1-5, デフォルト3
    pub related_articles: Vec<String>,         // 関連記事のslug配列
    pub tags: Vec<String>,                     // タグ配列
    pub created_at: Option<String>,            // 作成日（ISO 8601形式）
    pub updated_at: Option<String>,            // 更新日（ISO 8601形式）
}

impl Default for ArticleMetadata {
    fn default() -> Self {
        Self {
            title: "Untitled".to_string(),
            home_display: false,               // デフォルトは非表示
            category: None,
            importance: Some(3),               // デフォルト重要度
            related_articles: Vec::new(),
            tags: Vec::new(),
            created_at: None,
            updated_at: None,
        }
    }
}
```

#### Front Matterパーサー

```rust
pub struct FrontMatterParser;

impl FrontMatterParser {
    pub fn parse(content: &str) -> Result<(ArticleMetadata, String), ParseError> {
        // YAML front matterを解析
        // 記事本文とメタデータを分離
    }
    
    pub fn extract_yaml_block(content: &str) -> Option<&str> {
        // ---で囲まれたYAMLブロックを抽出
    }
    
    pub fn parse_yaml_metadata(yaml: &str) -> Result<ArticleMetadata, ParseError> {
        // YAMLをArticleMetadataに変換
    }
}
```

#### リンク解析システム

```rust
pub struct LinkExtractor;

impl LinkExtractor {
    pub fn extract_all_links(content: &str) -> Vec<ArticleLink> {
        let mut links = Vec::new();
        links.extend(Self::extract_wiki_links(content));
        links.extend(Self::extract_markdown_links(content));
        links.extend(Self::extract_tag_references(content));
        links
    }
    
    pub fn extract_wiki_links(content: &str) -> Vec<ArticleLink> {
        // [[記事名]]形式のリンクを抽出
        // 正規表現: \[\[([^\]]+)\]\]
    }
    
    pub fn extract_markdown_links(content: &str) -> Vec<ArticleLink> {
        // [テキスト](slug)形式のリンクを抽出
        // 正規表現: \[([^\]]+)\]\(([^)]+)\)
    }
    
    pub fn extract_tag_references(content: &str) -> Vec<ArticleLink> {
        // #tag形式のタグを抽出
        // 正規表現: #([a-zA-Z0-9_-]+)
    }
    
    pub fn get_link_context(content: &str, position: usize, context_length: usize) -> String {
        // リンク周辺のテキストを抽出
    }
}

pub struct LinkGraph {
    connections: HashMap<String, Vec<Connection>>, // slug -> connections
    tag_connections: HashMap<String, Vec<String>>, // tag -> slugs
    relationship_cache: HashMap<(String, String), f32>, // (from, to) -> strength
}

impl LinkGraph {
    pub fn new() -> Self;
    pub fn add_article(&mut self, article: &Article);
    pub fn remove_article(&mut self, slug: &str);
    pub fn get_connections(&self, slug: &str) -> Vec<&Connection>;
    pub fn calculate_relationship_strength(&self, from: &str, to: &str) -> f32;
    pub fn get_related_articles(&self, slug: &str, limit: usize) -> Vec<(String, f32)>;
    pub fn rebuild_cache(&mut self); // 関連度キャッシュを再構築
}

#[derive(Debug, Clone)]
pub struct Connection {
    pub target: String,                        // 接続先のslug
    pub connection_type: ConnectionType,       // 接続の種類
    pub strength: f32,                         // 接続の強さ（0.0-1.0）
    pub bidirectional: bool,                   // 双方向接続かどうか
}

#[derive(Debug, Clone, PartialEq)]
pub enum ConnectionType {
    DirectLink,     // 直接リンク
    TagBased,       // タグベースの関連
    Bidirectional,  // 双方向リンク
}
```

### Node Graph Data Model

```rust
pub struct NodeGraph {
    pub author_node: NodeId,
    pub article_nodes: HashMap<String, NodeId>, // slug -> NodeId
    pub connections: Vec<PhysicsConnection>,
    pub layout_state: LayoutState,
    pub link_graph: Rc<RefCell<LinkGraph>>,    // 記事間のリンク情報
}

impl NodeGraph {
    pub fn new(link_graph: Rc<RefCell<LinkGraph>>) -> Self;
    pub fn update_from_link_graph(&mut self);  // LinkGraphから物理接続を更新
    pub fn get_connection_strength(&self, from: NodeId, to: NodeId) -> f32;
    pub fn highlight_related_nodes(&self, node_id: NodeId) -> Vec<NodeId>;
}

pub struct PhysicsConnection {
    pub from: NodeId,
    pub to: NodeId,
    pub connection_type: PhysicsConnectionType,
    pub strength: f32,                         // 物理演算での接続強度
    pub visual_strength: f32,                  // 視覚的な線の太さ
    pub animated: bool,                        // アニメーション効果
}

pub enum PhysicsConnectionType {
    AuthorToArticle,
    DirectLink,        // 直接リンク
    TagBased,         // タグベースの関連
    Bidirectional,    // 双方向リンク
}

pub struct LayoutState {
    pub center_position: Position,
    pub zoom_level: f32,
    pub pan_offset: Position,
    pub highlighted_nodes: Vec<NodeId>,        // ハイライト中のノード
    pub active_connections: Vec<usize>,        // アクティブな接続のインデックス
}
```

### Configuration Data Model

```rust
pub struct PortfolioConfig {
    pub author: AuthorConfig,
    pub display: DisplayConfig,
    pub physics: PhysicsConfig,
}

pub struct AuthorConfig {
    pub name: String,
    pub image_url: String,
    pub bio: String,
    pub social_links: HashMap<String, String>,
}

pub struct DisplayConfig {
    pub max_home_articles: usize,
    pub default_node_size: i32,
    pub category_colors: HashMap<String, String>,
    pub animation_speed: f32,
}
```

## Error Handling

### Error Types

```rust
#[derive(Debug)]
pub enum PortfolioError {
    ArticleLoadError(String),
    MetadataParseError(String),
    PhysicsError(String),
    RenderError(String),
    ConfigurationError(String),
}

impl std::fmt::Display for PortfolioError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            PortfolioError::ArticleLoadError(msg) => write!(f, "Article load error: {}", msg),
            PortfolioError::MetadataParseError(msg) => write!(f, "Metadata parse error: {}", msg),
            PortfolioError::PhysicsError(msg) => write!(f, "Physics error: {}", msg),
            PortfolioError::RenderError(msg) => write!(f, "Render error: {}", msg),
            PortfolioError::ConfigurationError(msg) => write!(f, "Configuration error: {}", msg),
        }
    }
}
```

### Error Recovery Strategies

1. **記事読み込みエラー**: デフォルト記事を表示、エラーログを記録
2. **メタデータ解析エラー**: デフォルト値を使用、警告を表示
3. **物理演算エラー**: 物理演算を一時停止、静的レイアウトにフォールバック
4. **レンダリングエラー**: エラー境界でキャッチ、エラーメッセージを表示
5. **データ生成エラー**: 既存のJSONファイルを使用、エラー詳細をコンソール出力
6. **ローカル開発エラー**: フォールバックモードで最小限のデータセットを生成

### Local Development Workflow

#### 開発環境セットアップ
```bash
# 1. リポジトリクローン
git clone <repository-url>
cd portfolio

# 2. Nix開発環境に入る
nix develop

# 3. 初回データ生成
just build-data

# 4. 開発サーバー起動（記事監視付き）
just dev
```

#### 記事作成・編集ワークフロー
```bash
# Nix開発環境内で作業
nix develop

# 1. 新しい記事作成
touch articles/new-article.md

# 2. Front matterとコンテンツを記述
# ---
# title: "新しい記事"
# home_display: true
# category: "tech"
# ---

# 3. 記事処理を実行
just process-articles

# 4. 開発サーバーで確認（自動リロード）
just serve
```

#### デバッグとトラブルシューティング
```bash
# Nix環境内で実行
nix develop

# リンク検証のみ実行
just validate-links

# 詳細なデバッグ情報付きで処理
cargo run --bin process-articles -- --verbose

# 生成されたデータの確認
cat data/articles.json | jq '.summary'
cat data/validation-report.json | jq '.errors'

# キャッシュクリア
just clean && just build-data
```

#### GitHub Actionsとの同期
```bash
# ローカルでGitHub Actionsと同じ処理を実行
nix develop --command just build-data
nix develop --command just test

# 生成されたファイルをコミット
git add data/*.json
git commit -m "Update article data"
git push
```

#### Nix Shell使用例
```bash
# 一時的にNix環境を使用
nix shell

# 特定のコマンドのみNix環境で実行
nix develop --command just build

# CI環境の再現
nix develop --command bash -c "just build-data && just test && just build"
```

## Testing Strategy

### 1. Unit Tests

#### Article Management
- Markdownファイルの解析テスト
- Front matterのパースエラーハンドリング
- 記事フィルタリング機能

#### Physics System
- ノード間の力の計算
- 衝突検出
- 境界条件の処理

#### Node Rendering
- 各ノードタイプの描画
- カテゴリ別スタイリング
- レスポンシブレイアウト

### 2. Integration Tests

#### End-to-End Workflow
- 記事読み込み → ノード生成 → 物理演算 → 描画
- ユーザーインタラクション（ドラッグ、クリック）
- ルーティングとナビゲーション

#### Performance Tests
- 大量ノードでの物理演算性能
- メモリ使用量の監視
- レンダリング性能の測定

### 3. Visual Regression Tests

#### Layout Tests
- 異なる画面サイズでのレイアウト
- ノード配置の一貫性
- アニメーションの滑らかさ

### 4. Accessibility Tests

#### WCAG Compliance
- キーボードナビゲーション
- スクリーンリーダー対応
- カラーコントラスト

### Test Implementation Framework

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use wasm_bindgen_test::*;

    #[wasm_bindgen_test]
    fn test_article_loading() {
        // テスト実装
    }

    #[wasm_bindgen_test]
    fn test_node_physics() {
        // テスト実装
    }

    #[wasm_bindgen_test]
    fn test_responsive_layout() {
        // テスト実装
    }
}
```

## Performance Considerations

### 1. Physics Optimization
- 物理演算の更新頻度を動的に調整
- 静止状態のノードの計算をスキップ
- 空間分割による衝突検出の最適化

### 2. Rendering Optimization
- 仮想DOM差分の最小化
- Canvas/WebGLレンダリングの検討
- アニメーションのrequestAnimationFrame使用

### 3. Memory Management
- 不要なノードデータの解放
- 物理世界の定期的なクリーンアップ
- 記事データの遅延読み込み

### 4. Bundle Size Optimization
- 未使用コードの除去
- 動的インポートの活用
- WebAssemblyバイナリの最適化