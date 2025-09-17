# Interactive Mindmap Portfolio - Task Runner

# Default recipe
default:
    @just --list

# Start development environment
dev:
    @echo "🚀 Starting development environment..."
    @just process-articles
    @cd khimoo-portfolio && trunk serve

# Process all articles
process-articles:
    @echo "📝 Processing articles..."
    @cd khimoo-portfolio && cargo run --bin process-articles --features cli-tools

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

# Check code formatting
fmt:
    @echo "🎨 Formatting code..."
    @cargo fmt

# Run clippy lints
clippy:
    @echo "📎 Running clippy..."
    @cargo clippy -- -D warnings

# Full development workflow
full-dev: fmt clippy test build-data
    @echo "🎯 Full development check complete"
    @trunk serve