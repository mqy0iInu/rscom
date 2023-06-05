# PJ進捗
`進捗率` ... **37.50%**  

<img src="dev/pj_status.png" alt="file">

# （TBD）エミュレータの構成
- NES
  - WRAM
  - PPU
    - VARM
    - 音声出力
  - RP2A03
    - CPU
        - 命令デコーダ
        - ALU
        - CPUレジスタ
            - アキュムレータ(汎用レジスタ)
            - インデックスレジスタ X
            - インデックスレジスタ X
            - ステータスレジスタ
            - SP
            - PC
      - 割り込みコントローラ
        - RST
        - NMI
        - IRQ
      - バスコントローラ
      - GCG(クロックジェネレータ)
      - DMA
      - APU
      - GPIO
      - タイマー(タイミングコントローラ)

# 参考文献
下記に参考文献を示す。

## 6502 & RP2A03 リファレンス
https://www.nesdev.org/wiki/NES_reference_guide
https://www.nesdev.org/obelisk-6502-guide/reference.html
https://pgate1.at-ninja.jp/NES_on_FPGA/

## Rust リファレンス(公式)
https://doc.rust-jp.rs/book-ja/
https://doc.rust-jp.rs/
https://doc.rust-jp.rs/rust-by-example-ja/
https://doc.rust-lang.org/stable/std/index.html

## Rust リファレンス(Microsoft)
https://learn.microsoft.com/ja-jp/training/modules/rust-introduction/2-rust-overview

## Rust リファレンス(有志)
https://sinkuu.github.io/api-guidelines/naming.html
https://makandat.wordpress.com/2022/02/05/rust-%E3%81%AE%E5%8B%89%E5%BC%B7-snake-case-name/
https://zenn.dev/mebiusbox/books/22d4c1ed9b0003/viewer/6d5875
https://zenn.dev/tfutada/articles/16766e3b4560db
https://zenn.dev/hankei6km/articles/using-jemalloc-in-rust-speeds-up-parallelism
https://zenn.dev/khale/articles/rust-beginners-catchup
https://qiita.com/yoshii0110/items/6d70323f01fefcf09682

## 組み込みRust リファレンス(有志)
https://tomoyuki-nakabayashi.github.io/book/intro/index.html
https://qiita.com/ochaochaocha3/items/1969d76debd6d3b42269
https://lab.seeed.co.jp/entry/2021/04/30/180000

## 環境構築
https://qiita.com/yannori/items/189cc0dbce2b81b9d1e1
https://zenn.dev/watarukura/articles/20220304-8nefpx6tlmhxgbpvqwah2gzoff
https://zenn.dev/fah_72946_engr/articles/cf53487d3cc5fc