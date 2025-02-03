use std::fs::OpenOptions;
use std::io::{Read, Seek, SeekFrom, Write};
use std::path::PathBuf;
use std::sync::Mutex;
use crate::storage::page::Page;

#[derive(Debug, Clone)]
pub struct BlockId {
    pub filename: PathBuf,
    pub number: u32,
}

impl BlockId {
    /// BlockId を作成します。  
    /// ※ filename は &str だけでなく、PathBuf も受け付けます。
    pub fn new<P: Into<PathBuf>>(filename: P, number: u32) -> BlockId {
        BlockId {
            filename: filename.into(),
            number,
        }
    }
}

/// FileManager クラス
/// - db_directory と block_size をプライベート変数に持ちます。
/// - 同時実行を防ぐため、内部に Mutex を保持します。
pub struct FileManager {
    db_directory: PathBuf,
    block_size: usize,
    lock: Mutex<()>,
}

impl FileManager {
    /// 新しい FileManager を作成します。
    /// - `db_directory`: データベースのディレクトリ（ファイル群の置かれているディレクトリ）
    /// - `block_size`: ブロックのサイズ（バイト単位）
    pub fn new<P: Into<PathBuf>>(db_directory: P, block_size: usize) -> FileManager {
        FileManager {
            db_directory: db_directory.into(),
            block_size,
            lock: Mutex::new(()),
        }
    }
    
    /// 指定された BlockId のブロックをファイルから読み込み、Page にセットします。
    /// このメソッドは Mutex によって排他的に実行されるため、
    /// 複数のスレッドで同時に呼び出されても一度に一つしか実行されません。
    pub fn read(&self, block: &BlockId, page: &mut Page) -> std::io::Result<()> {
        // Mutex をロックして排他制御
        let _guard = self.lock.lock().unwrap();

        // db_directory と BlockId.filename を結合してファイルのフルパスを作成
        let mut path = self.db_directory.clone();
        path.push(&block.filename);
        
        // ファイルをオープン
        let mut file = std::fs::File::open(&path)?;
        
        // ブロックの先頭オフセットを計算 (block_size * block.number)
        let offset = (self.block_size as u64) * (block.number as u64);
        file.seek(SeekFrom::Start(offset))?;
        
        // block_size バイト分のデータを読み込む
        let mut buffer = vec![0u8; self.block_size];
        let n = file.read(&mut buffer)?;
        if n != self.block_size {
            return Err(std::io::Error::new(
                std::io::ErrorKind::UnexpectedEof,
                "Could not read full block",
            ));
        }
        
        // 読み込んだデータを Page にセット（読み出し位置は 0 にリセット）
        page.write_bytes(buffer.as_slice());
        
        // _guard はスコープ終了時に自動的に解放されます。
        Ok(())
    }

    /// write(block, page)
    /// Page の内容を、BlockId が示すブロック位置に書き込みます。
    pub fn write(&self, block: &BlockId, page: &Page) -> std::io::Result<()> {
        // 排他制御
        let _guard = self.lock.lock().unwrap();

        // db_directory と BlockId.filename を結合してファイルパスを作成
        let mut path = self.db_directory.clone();
        path.push(&block.filename);
        
        // 書き込みモードでファイルをオープン（ファイルは既存のものとする）
        let mut file = OpenOptions::new().write(true).open(&path)?;
        let offset = (self.block_size as u64) * (block.number as u64);
        file.seek(SeekFrom::Start(offset))?;
        file.write(&page.bytebuffer())?;
        Ok(())
    }
    
    /// append(filename)
    /// 指定されたファイル名に対して、新たなブロックを確保（ファイルサイズを block_size 分延長）し、
    /// そのブロックの BlockId を返します。
    pub fn append(&self, filename: String) -> std::io::Result<BlockId> {
        // 排他制御
        let _guard = self.lock.lock().unwrap();
        
        let mut path = self.db_directory.clone();
        path.push(&filename);
        
        // ファイルを読み書き可能な状態でオープン（存在しなければ作成）
        let mut file = OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .open(&path)?;
        
        // 現在のファイルサイズを取得
        let file_len = file.metadata()?.len();
        // 現在のブロック数＝ファイルサイズ / block_size（余りは無視）
        let block_number = (file_len / (self.block_size as u64)) as u32;
        // 新たなブロック分、ファイルサイズを延長する
        let new_len = file_len + self.block_size as u64;
        file.set_len(new_len)?;
        
        // 確保したブロックの BlockId を返す
        Ok(BlockId::new(filename, block_number))
    }    
}
