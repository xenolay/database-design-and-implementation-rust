use std::path::PathBuf;

#[derive(Debug, Clone)]
pub struct BlockId {
    pub filename: PathBuf,
    pub number: u32,
}

impl BlockId {
    /// 新しい BlockId を作成します。
    /// ここでは filename に対して Into<PathBuf> を使うことで、
    /// &str や PathBuf などを柔軟に受け付けます。
    pub fn new<P: Into<PathBuf>>(filename: P, number: u32) -> BlockId {
        BlockId {
            filename: filename.into(),
            number,
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::storage::block_id::BlockId;

    #[test]
    fn it_works() {
        let filename: &str = "testfile";
        let number = 123;

        let blockid = BlockId::new(filename, number);
        assert_eq!(blockid.filename.as_os_str(), "testfile");
        assert_eq!(blockid.number, number);
    }
}