# エラーログ（過去のハマりと解決の原文）

> 2 回以上試行した問題と解決法の詳細記録。要点だけ拾うなら `known-issues.md` を参照。

## 1. WSL を `bash -lc '...'` でインライン実行すると `$VAR`/`$(...)` が消える

**うまくいかなかったこと:**
PowerShell→Bashツール→`wsl bash -lc '...'` の多重ネストで、シングルクオート内の
`for M in ...; do ... $M ...` や `D=$(mktemp -d)` の変数展開が空になり、
`scripts/roundtrip.sh` 相当のラウンドトリップ検証が全件MISMATCHになる
（コマンド自体は成立するため気づきにくい）。

**うまくいったこと:**
テスト内容を `.sh` ファイルに書き出し、`wsl bash -lc 'bash /mnt/d/.../script.sh'` で実行。
スクリプト内なら変数・コマンド置換が正常に効く。
注意: `/mnt/...` を `bash` の引数に直接渡すと Git Bash がパス変換するため、
`-lc` 文字列の**中**に `/mnt` パスを置くこと。

**次回のために:**
WSL越しに変数・ループ・コマンド置換を含む処理を流すときは、インラインでなく
スクリプトファイル化する。`scripts/roundtrip.sh` が既にこの形なのもそのため。
（本問題は `tmg1-codec` 側でも発見された。同内容を `tmg1-codec` の errors-log.md にも記録）

## 2. submodule のローカル clone の remote が旧ホストのまま残る

**うまくいかなかったこと:**
`tmg1-codec` submodule のポインタを最新へ上げようと配下で `git fetch origin` しても
目的のコミットが取れず `fatal: unable to read tree <sha>` / `Not a valid object name`。

**うまくいったこと:**
`git submodule sync tmg1-codec` で `.gitmodules` の URL を submodule の remote へ反映してから
fetch。superproject の `.gitmodules` を変更しただけでは既存 submodule の `.git/config` remote は
自動追随しない。

**次回のために:**
submodule の参照先ホストを変更した後は、必ず `git submodule sync` を先に実行する。
