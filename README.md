# candfans-downloader

[Candfans](https://candfans.jp) からコンテンツをざっとダウンロードするためのもの。

```console
candfans-downloader <TARGET_USER> -c <COOKIE> -x <X-XSRF-TOKEN> [-O <OUTPUT_DIR>]
```

- `TARGET_USER` は `https://candfans.jp/<TARGET_USER>` でアクセスできる任意の ID を設定する
- `COOKIE` および `X-XSRF-TOKEN` は Candfans にログインしている状態で `https://candfans.jp/api/*` にアクセスした際に付与されるリクエストヘッダーより取得する

---

Thanks [candfans-dl](https://github.com/xdnauly/candfans-dl)
