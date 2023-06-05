# NES Emulator by Rust / ファミコンエミュレータ by Rust
`Rustの勉強`で`『習うより慣れろ』`でファミコンを作成中　(≧▽≦) /  
マイコン畑なC/C++の組み込みさんのRust奮闘記www(仮)  
  
`★売りポイント` ... `並列、並行処理! マルチスレッドなファミコンエミュレータ`

# PJ進捗（個人PJですこれw）
`進捗率` ... **37.50%**  
※個人PJやのに、進捗管理しちゃうのはPMの職業病www

<img src="dev/pj_status.png" alt="file">

# エミュレータの構成(設計内容)

- メイン関数
  - メインループ
  - CPUスレッド
    - 1 命令フェッチ
    - 2 命令デコード
    - 3 命令実行
  - PPUスレッド
  - APUスレッド

# 並列、並行！マルチスレッド
組み込み屋はマイコン畑でFreeRTOSかITRONのRTOS使いさんなので、  
並列、並行処理なマルチスレッド（つい、マルチタスクって出るｗ）は朝飯前！

※下記はスレッドしてるよー！っていう、実際のmain.rsの中身！

```Rust:main.rs
fn main()
{
// ==================================================================================
    // [H/W Reset & App Init]
    app_init();

// ==================================================================================
// [Thred Main Loop]
    let _cpu_thread = thread::spawn(|| {
        loop {
            cpu_main();
            thread::sleep(Duration::from_millis(300));
        }
    });

    let _ppu_thread = thread::spawn(|| {
        loop {
            ppu_main();
            thread::sleep(Duration::from_millis(500));
        }
    });

    let _apu_thread = thread::spawn(|| {
        loop {
            apu_main();
            thread::sleep(Duration::from_millis(800));
        }
    });

// ==================================================================================
// [Main Loop]
    loop {
        println!("[DEBUG] : App Main Loop");
        thread::sleep(Duration::from_millis(999));
    }
// ==================================================================================
}
```
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