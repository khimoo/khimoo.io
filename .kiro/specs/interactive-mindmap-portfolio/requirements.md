# Requirements Document

## Introduction

インタラクティブなマインドマップ形式のポートフォリオサイトを構築します。ユーザーの写真を中心に配置し、そこから関連する記事やページへのノードが物理演算によって動的に配置されます。Obsidianのようなマインドマップ形式で、記事間の関連性を視覚的に表現し、ユーザーが直感的にコンテンツを探索できるインタラクティブな体験を提供します。

## 重要な開発環境の注意事項

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

## Requirements

### Requirement 1

**User Story:** ポートフォリオの訪問者として、中央に配置された作者の写真を起点として、関連する記事やページを視覚的に探索したい。そうすることで、作者の専門性や興味分野を直感的に理解できる。

#### Acceptance Criteria

1. WHEN ホーム画面が読み込まれる THEN システムは作者の写真を画面中央に表示する SHALL
2. WHEN ホーム画面が表示される THEN システムは選択された記事ノードを作者の写真周辺に配置する SHALL
3. WHEN ユーザーがノードをクリックする THEN システムは対応する記事ページに遷移する SHALL

### Requirement 2

**User Story:** ポートフォリオの管理者として、全ての記事の中から特定の記事のみをホーム画面に表示したい。そうすることで、記事数が増加しても核となる重要な記事のみを強調して表示できる。

#### Acceptance Criteria

1. WHEN 記事にhome_display: trueのメタデータが設定されている THEN システムはその記事をホーム画面のノードとして表示する SHALL
2. WHEN 記事にhome_display: falseまたはメタデータが未設定の場合 THEN システムはその記事をホーム画面に表示しない SHALL
3. WHEN 管理者が記事のメタデータを更新する THEN システムはホーム画面の表示を動的に更新する SHALL

### Requirement 3

**User Story:** ポートフォリオの訪問者として、関連性のある記事同士が視覚的に繋がって表示されることを期待する。そうすることで、記事間の関係性を理解し、関連するコンテンツを効率的に発見できる。

#### Acceptance Criteria

1. WHEN 記事内に`[[記事名]]`形式のリンクが存在する THEN システムは自動的に記事間の関連性を検出する SHALL
2. WHEN 記事内に`[テキスト](記事slug)`形式のリンクが存在する THEN システムは記事間の接続を認識する SHALL
3. WHEN 関連記事同士がホーム画面に表示されている THEN システムは両ノード間に視覚的な接続線を表示する SHALL
4. WHEN ユーザーが記事内のリンクをクリックする THEN システムは対応する記事ノードにフォーカスして遷移する SHALL

### Requirement 4

**User Story:** ポートフォリオの訪問者として、ノードが物理演算によって自然に動作することを期待する。そうすることで、インタラクティブで魅力的なユーザー体験を得られる。

#### Acceptance Criteria

1. WHEN ユーザーがノードをドラッグする THEN システムはノードを物理演算に基づいて移動させる SHALL
2. WHEN ノードが他のノードに近づく THEN システムは反発力を適用してノード同士の重複を防ぐ SHALL
3. WHEN ノードが中心から離れる THEN システムは中心への引力を適用してレイアウトを維持する SHALL
4. WHEN 関連記事間に接続がある THEN システムはスプリング力を適用して適切な距離を保つ SHALL

### Requirement 5

**User Story:** ポートフォリオの管理者として、Obsidianのような直感的な方法でMarkdown記事を作成・管理したい。そうすることで、記事間の関連性を自然に表現し、効率的にコンテンツを組織化できる。

#### Acceptance Criteria

1. WHEN 管理者がarticlesディレクトリにMarkdownファイルを配置する THEN システムはそのファイルを記事として認識する SHALL
2. WHEN Markdownファイルにfront matterが含まれる THEN システムはメタデータを解析して適用する SHALL
3. WHEN 管理者が記事内で`[[記事名]]`形式のリンクを記述する THEN システムは自動的に記事間の関連性を検出する SHALL
4. WHEN 記事間に双方向リンクが存在する THEN システムは両記事に相互参照を表示する SHALL
5. WHEN ユーザーが記事ページにアクセスする THEN システムはMarkdownをHTMLに変換し、内部リンクを適切に処理して表示する SHALL
6. WHEN 管理者がfront matterにtagsフィールドを設定する THEN システムはタグ情報をメタデータとして記録する SHALL

### Requirement 6

**User Story:** ポートフォリオの訪問者として、レスポンシブなデザインでモバイルデバイスからも快適に閲覧したい。そうすることで、デバイスを問わずポートフォリオを探索できる。

#### Acceptance Criteria

1. WHEN ユーザーがモバイルデバイスでアクセスする THEN システムはタッチ操作に対応したインターフェースを提供する SHALL
2. WHEN 画面サイズが変更される THEN システムはノードレイアウトを画面サイズに適応させる SHALL
3. WHEN ユーザーがピンチジェスチャーを行う THEN システムはズーム機能を提供する SHALL

### Requirement 7

**User Story:** ポートフォリオの訪問者として、ノードの視覚的な表現が記事の重要度を反映していることを期待する。そうすることで、どの記事が重要かを直感的に理解できる。

#### Acceptance Criteria

1. WHEN 記事にimportanceレベル（1-5）が設定されている THEN システムはノードのサイズを重要度に応じて調整する SHALL
2. WHEN 記事が多くの他記事からリンクされている THEN システムはインバウンドリンク数に応じてノードサイズを動的に調整する SHALL
3. WHEN ノードサイズが計算される THEN システムは重要度とリンク数を組み合わせて15px-60pxの範囲でサイズを決定する SHALL
4. WHEN 記事間に直接リンクが存在する THEN システムは既存の接続線表示機能を使用して視覚的な接続を表示する SHALL
5. WHEN デバッグモードが有効な場合 THEN システムはノード間の結合力を調整可能なUIを提供する SHALL

### Requirement 8

**User Story:** ポートフォリオの管理者として、記事間のリンク関係を視覚的に確認・管理したい。そうすることで、コンテンツの構造を把握し、適切な情報アーキテクチャを維持できる。

#### Acceptance Criteria

1. WHEN 記事間にリンクが作成される THEN システムは双方向の関連性を自動的に認識する SHALL
2. WHEN 記事が削除される THEN システムは他記事からの参照リンクを検出してコンソールに警告を出力する SHALL
3. WHEN 管理者がリンク切れを確認したい THEN システムは存在しない記事への参照を検出してコンソールに報告する SHALL
4. WHEN アプリケーション起動時 THEN システムは全記事のリンク整合性をチェックして問題があれば報告する SHALL