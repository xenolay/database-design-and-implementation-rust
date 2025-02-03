pub struct Page {
    // バッファは外部から直接アクセスできないよう private にしておく
    bytebuffer: Vec<u8>,
    // 現在の読み書き位置（バッファ内のインデックス）
    pos: usize,
}

impl Page {
    /// 指定した容量で新しい Page を作成します。
    pub fn new(capacity: usize) -> Self {
        Page {
            bytebuffer: Vec::with_capacity(capacity),
            pos: 0,
        }
    }

    /// i32 の値を 4 バイト（ビッグエンディアン形式）に変換して書き込みます。
    pub fn write_int(&mut self, value: i32) {
        let bytes = value.to_be_bytes();
        self.write_bytes(&bytes);
    }

    /// 1 バイトを書き込みます。
    pub fn write_byte(&mut self, value: u8) {
        if self.pos < self.bytebuffer.len() {
            // すでに存在する位置なら上書き
            self.bytebuffer[self.pos] = value;
        } else {
            // それ以外は末尾に追加
            self.bytebuffer.push(value);
        }
        self.pos += 1;
    }

    /// &str を書き込みます。  
    /// まず文字列のバイト数（i32）を書き、続いて UTF-8 のバイト列を書き込みます。
    pub fn write_str(&mut self, value: &str) {
        let bytes = value.as_bytes();
        let len = bytes.len() as i32;
        self.write_int(len);
        self.write_bytes(bytes);
    }

    /// 与えられたバイト列を順次書き込みます。
    pub fn write_bytes(&mut self, bytes: &[u8]) {
        for &b in bytes {
            self.write_byte(b);
        }
    }

    /// 読み込み用に内部位置を 0 に戻します。  
    /// （書き込み後、バッファ先頭から読み出すときに利用）
    pub fn flip(&mut self) {
        self.pos = 0;
    }

    /// 現在の位置から 4 バイトを読み出し、i32（ビッグエンディアン）に変換して返します。
    /// 読み出しできない場合は None を返します。
    pub fn read_int(&mut self) -> Option<i32> {
        if self.pos + 4 > self.bytebuffer.len() {
            return None;
        }
        let slice = &self.bytebuffer[self.pos..self.pos + 4];
        self.pos += 4;
        Some(i32::from_be_bytes([slice[0], slice[1], slice[2], slice[3]]))
    }

    /// 現在の位置から 1 バイトを読み出します。
    pub fn read_byte(&mut self) -> Option<u8> {
        if self.pos >= self.bytebuffer.len() {
            return None;
        }
        let value = self.bytebuffer[self.pos];
        self.pos += 1;
        Some(value)
    }

    /// 現在の位置から文字列を読み出します。  
    /// まず先頭の 4 バイトで文字列の長さ（i32）を読み、その後その長さ分のバイトを取り出して UTF-8 の文字列に変換します。
    pub fn read_str(&mut self) -> Option<String> {
        let len = self.read_int()? as usize;
        if self.pos + len > self.bytebuffer.len() {
            return None;
        }
        let slice = &self.bytebuffer[self.pos..self.pos + len];
        self.pos += len;
        std::str::from_utf8(slice).map(|s| s.to_string()).ok()
    }

    // 外部には公開しないアクセサ
    pub(in crate::storage) fn bytebuffer(&self) -> &Vec<u8> {
        &self.bytebuffer
    }
}
