# Requirements Document

## Introduction

ポートフォリオサイトの基本的なCI/CDシステムをGitHub Actionsで構築します。記事をGitHubリポジトリにpushした際に、記事間のリンク関係性情報を抽出し、ポートフォリオページを自動的にビルド・デプロイすることを目的とします。

**技術的前提**
- Nix環境による再現可能なビルド環境を活用し、ローカル開発環境とCI環境の一貫性を確保します
- flake.nixで定義された依存関係により、環境の違いによる問題を回避します

## Requirements

### Requirement 1

**User Story:** ポートフォリオの管理者として、記事ファイルをGitHubリポジトリにpushした際に自動的に記事処理が実行されることを期待する。そうすることで、記事間のリンク関係性情報が抽出され、最新のデータでポートフォリオが更新される。

#### Acceptance Criteria

1. WHEN mainブランチにpushされる THEN システムは自動的にGitHub Actionsワークフローを開始する SHALL
2. WHEN ワークフローが実行される THEN システムはflake.nixで定義されたNix環境内でcargo run --bin process-articlesを実行する SHALL
3. WHEN 記事処理が完了する THEN システムは記事間のリンク関係性を含むarticles.jsonとlink-graph.jsonを生成する SHALL

### Requirement 2

**User Story:** ポートフォリオの管理者として、記事処理後に自動的にWebサイトがビルド・デプロイされることを期待する。そうすることで、変更が即座に本番環境に反映され、手動デプロイの手間を省ける。

#### Acceptance Criteria

1. WHEN 記事処理が完了する THEN システムは自動的にWebAssemblyビルドを開始する SHALL
2. WHEN WebAssemblyビルドが実行される THEN システムはNix環境内でtrunk buildを実行する SHALL
3. WHEN ビルドが完了する THEN システムは生成されたdist/ディレクトリをGitHub Pagesにデプロイする SHALL
4. WHEN デプロイが完了する THEN システムはポートフォリオサイトを新しいURLでアクセス可能にする SHALL

### Requirement 3

**User Story:** ポートフォリオの管理者として、CI/CDシステムが効率的で保守しやすいことを期待する。そうすることで、ビルド時間を短縮し、トラブルシューティングを容易にできる。

#### Acceptance Criteria

1. WHEN GitHub Actionsワークフローが設計される THEN システムは必要最小限の処理のみを実行する SHALL
2. WHEN 複雑な処理が必要な場合 THEN システムはローカル環境で事前に検証してからCI環境に統合する SHALL
3. WHEN エラーが発生する THEN システムは簡潔で分かりやすいエラーメッセージを提供する SHALL
4. WHEN ビルドプロセスが実行される THEN システムはビルド時間の最適化を重視する SHALL

### Requirement 4

**User Story:** ポートフォリオの管理者として、GitHub ActionsがGitHub Pagesに正常にデプロイできることを期待する。そうすることで、権限エラーやデプロイ失敗を回避し、既存の動作するデプロイメント方式を活用できる。

#### Acceptance Criteria

1. WHEN GitHub Pagesデプロイが実行される THEN システムは既存のgh-pages.ymlで動作していたデプロイメント方式を基準とする SHALL
2. WHEN デプロイプロセスが設計される THEN システムはpeaceiris/actions-gh-pages@v3アクションを使用し、publicディレクトリ構造を維持する SHALL
3. WHEN ビルド成果物が準備される THEN システムはdist/ディレクトリをpublic/ディレクトリにコピーし、404.htmlを適切に設定する SHALL
4. WHEN 権限エラーが発生する THEN システムは既存の動作するワークフローの権限設定を参考にして問題を解決する SHALL

