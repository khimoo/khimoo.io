# Requirements Document

## Introduction

現在のポートフォリオシステムでは、作者ノードがcomponents.rsでID=0としてハードコーディングされています。この仕様では、作者ノードの管理方法をメタデータベースのアプローチに変更し、markdownファイルのメタデータで`author_image`フィールドを指定することで作者ノードを定義できるようにします。`author_image`フィールドが存在するファイルが作者ノードとして認識され、同時に顔写真も表示されます。既存のコードへの影響を最小限に抑えながら、より柔軟で保守性の高いシステムを構築することを目的とします。

## Requirements

### Requirement 1

**User Story:** 開発者として、作者ノードをハードコーディングではなくメタデータで定義したいので、より柔軟で保守性の高いシステムにしたい

#### Acceptance Criteria

1. WHEN markdownファイルのメタデータに`author_image`フィールドが設定されている THEN そのファイルが作者ノードとして認識される SHALL システム
2. WHEN 複数のファイルに`author_image`フィールドが設定されている THEN 最初に見つかったファイルが作者ノードとして使用される SHALL システム
3. WHEN `author_image`フィールドが設定されたファイルが存在しない THEN 従来のID=0のハードコーディングされた作者ノードが使用される SHALL システム
4. WHEN 作者ノードが定義されている THEN そのファイルのタイトルが作者ノードの表示名として使用される SHALL システム

### Requirement 2

**User Story:** 開発者として、作者ノードに顔写真を表示したいので、より個人的で親しみやすいポートフォリオにしたい

#### Acceptance Criteria

1. WHEN 作者ノードのmarkdownファイルのメタデータに`author_image`フィールドが設定されている THEN その画像パスが作者ノードで使用される SHALL システム
2. WHEN 作者ノードに画像が設定されている THEN ノードのdiv要素内にその画像が表示される SHALL システム
3. WHEN 作者ノードに画像が設定されていない THEN テキストベースの表示が使用される SHALL システム
4. WHEN 画像ファイルが存在しない THEN エラーを表示せずにテキストベースの表示にフォールバックする SHALL システム

### Requirement 3

**User Story:** 開発者として、既存のコードへの影響を最小限に抑えたいので、安全で段階的な移行ができるようにしたい

#### Acceptance Criteria

1. WHEN 新しいメタデータベースの作者ノードシステムが実装される THEN 既存のNodeRegistry、PhysicsWorld、NodeComponentの基本構造は変更されない SHALL システム
2. WHEN 新しいシステムが導入される THEN 既存の記事ノードの表示や動作に影響を与えない SHALL システム
3. WHEN メタデータの読み込みに失敗する THEN 従来のハードコーディングされた作者ノードが使用される SHALL システム
4. WHEN 新しいシステムが動作する THEN 既存の物理シミュレーションやドラッグ機能は正常に動作する SHALL システム

### Requirement 4

**User Story:** 開発者として、作者ノードの画像表示を適切にスタイリングしたいので、他のノードと一貫性のあるデザインにしたい

#### Acceptance Criteria

1. WHEN 作者ノードに画像が表示される THEN 画像は円形にクロップされて表示される SHALL システム
2. WHEN 作者ノードの画像が表示される THEN 画像のサイズはノードの動的サイズに合わせて調整される SHALL システム

### Requirement 5

**User Story:** 開発者として、メタデータの構造を明確に定義したいので、一貫性のある実装ができるようにしたい

#### Acceptance Criteria

1. WHEN 作者ノードのメタデータが定義される THEN `author_image: "path/to/image.jpg"`フィールドが必須である SHALL システム
2. WHEN 作者ノードが識別される THEN `author_image`フィールドの存在のみで作者ノードとして認識される SHALL システム
3. WHEN メタデータが解析される THEN 既存のメタデータフィールド（importance、home_displayなど）と互換性がある SHALL システム
4. WHEN 作者ノードが識別される THEN 他の記事ノードと同じデータ構造を使用する SHALL システム