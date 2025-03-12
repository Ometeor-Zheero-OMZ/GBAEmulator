# Descriptions

## 想定するファイル

```
gba/
├── Cargo.toml
├── src/
│   ├── main.rs       # エントリーポイント
│   ├── cpu.rs        # ARM7TDMIのCPUエミュレーション
│   ├── memory.rs     # メモリマネジメント（WRAM, VRAM, OAM, I/Oレジスタなど）
│   ├── bus.rs        # CPUとメモリの橋渡し
│   ├── gpu.rs        # PPU（グラフィック処理）
│   ├── audio.rs      # APU（サウンド処理）
│   ├── input.rs      # ユーザー入力処理
│   ├── timer.rs      # タイマー処理
│   ├── dma.rs        # DMA（ダイレクトメモリアクセス）
│   ├── rom.rs        # ROMの読み込みと管理
│   ├── debugger.rs   # デバッグ機能
│   ├── utils.rs      # 補助的な関数
│   ├── config.rs     # 設定管理
│   └── lib.rs        # モジュールのエクスポート
├── assets/           # BIOSやテストROMファイル
├── tests/            # 統合テスト
├── README.md
└── .gitignore
```

## セーブ方法について

GBA のセーブ方法には 3 種類ある。

- SRAM
- Flash
- EEPROM

### SRAM (Static RAM)

- バッテリーバックアップ方式のセーブ
- 容量：32KB (256Kbit)
- 一般的なゲーム：ポケットモンスタールビー・サファイアなど
- SRAM の特徴
  - 直接アドレス空間にマッピングされる
  - `0x0E000000 - 0x0E007FFF` の範囲に存在
  - 書き込み速度は速いが、電源を切ると消えるため **バッテリーバックアップ** が必要

### Flash Memory

- フラッシュメモリ方式のセーブ
- 容量：64KB (512Kbit) または 128KB (1Mbit)
- 一般的なゲーム：ポケットモンスターファイアーレッド・リーフグリーンなど
- Flash の特徴
  - 読み書きには **特定のコマンドシーケンス** が必要
  - `0x0E000000 - 0x0E00FFFF` (64KB) または `0x0E000000 - 0x0E01FFFF` (128KB) の範囲に存在
  - バッテリー不要
  - 書き込み速度は遅いが、**電源を切ってもデータが消えない**

### EEPROM (Electrically Erasable Programmable Read-Only Memory)

- EEPROM 方式のセーブ
- 容量：512B ～ 8KB
- 一般的なゲーム：ファイナルファンタジータクティクスアドバンスなど
- EEPROM の特徴
  - 通常のメモリマッピングではなく、**シリアル通信 (I2C ライク) でやり取り**
  - `0x0D000000 - 0x0DFFFFFF` の範囲を介してアクセス
  - バッテリー不要
  - 書き込み速度が遅い
