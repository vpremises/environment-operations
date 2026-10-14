# 環境の運用製品

[構成・実行例](README.md)。製品CLI、合成credential playground、Windows helperを配置しました。汎用部品はCrowsiの正本を参照し、秘密・実環境登録・timerを汎用SDKへ移しません。

全3 packageのpublish=falseを維持します。公開registry版の検証まではcredential SDKを固定Git revisionで参照し、兄弟checkoutやprivate registryは要求しません。native公開・Windows/WSL実受入・実環境への配置は別の判断対象です。

Dependency updates: [repository policy](docs/dependency-updates.md).
