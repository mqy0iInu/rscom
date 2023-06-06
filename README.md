# NES Emulator by Rust / ファミコンエミュレータ by Rust
`Rustの勉強`で`『習うより慣れろ』`でファミコンをエミュレータ作成中　(≧▽≦) /  
※ マイコン畑なC/C++の組み込み屋さんのRust奮闘記でもあるwww  

# エミュレータの構成(設計内容)
`自己PR` ... `並列、並行処理! マルチスレッドなファミコンエミュレータ`

- `メイン関数`
  - `CPUスレッド`
    - `1 命令フェッチ`
    - `2 命令デコード`
    - `3 命令実行`
    - `4 PPU,APUレジスタのポーリング`
    - `5 キーボード入力のポーリング(SDL2)`
  - `PPUスレッド`
    - `画面描画(SDL2)`
  - `APUスレッド`
    - `音声出力(SDL2)`
  - `メイン関数ループ`
    - (TBD)一時停止、巻き戻しなど

# PJ Status / PJ進捗状況
`進捗率` ... `43.75%`  

<img src="dev/pj_status.png" alt="file">

# Reference / 参考文献
下記に参考文献を示す。

## Block Diagram Reference
👇Very Very Nice! Block Diagram!👇

<img src="https://www.zupimages.net/up/20/35/rswa.png" alt="nes">

> 引用元(Reference From): https://forums.nesdev.org/viewtopic.php?t=20685&start=75

## 6502 & RP2A03 Reference
https://bugzmanov.github.io/nes_ebook/index.html  
https://www.nesdev.org/wiki/NES_reference_guide  
https://www.nesdev.org/obelisk-6502-guide/reference.html  
https://pgate1.at-ninja.jp/NES_on_FPGA/  

## Rust Reference(公式)
https://doc.rust-jp.rs/book-ja/  
https://doc.rust-jp.rs/  
https://doc.rust-jp.rs/rust-by-example-ja/  
https://doc.rust-lang.org/stable/std/index.html  

## Rust Reference
https://learn.microsoft.com/ja-jp/training/modules/rust-introduction/2-rust-overview  
https://sinkuu.github.io/api-guidelines/naming.html  
https://makandat.wordpress.com/2022/02/05/rust-%E3%81%AE%E5%8B%89%E5%BC%B7-snake-case-name/  
https://zenn.dev/mebiusbox/books/22d4c1ed9b0003/viewer/6d5875  
https://zenn.dev/tfutada/articles/16766e3b4560db  
https://zenn.dev/hankei6km/articles/using-jemalloc-in-rust-speeds-up-parallelism  
https://zenn.dev/khale/articles/rust-beginners-catchup  
https://qiita.com/yoshii0110/items/6d70323f01fefcf09682  

## 組み込みRust Reference(有志)
https://tomoyuki-nakabayashi.github.io/book/intro/index.html  
https://qiita.com/ochaochaocha3/items/1969d76debd6d3b42269  
https://lab.seeed.co.jp/entry/2021/04/30/180000  

## 環境構築
https://qiita.com/yannori/items/189cc0dbce2b81b9d1e1  
https://zenn.dev/watarukura/articles/20220304-8nefpx6tlmhxgbpvqwah2gzoff  
https://zenn.dev/fah_72946_engr/articles/cf53487d3cc5fc  