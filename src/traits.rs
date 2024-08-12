pub trait WriteBuf {
    fn write(&mut self, data: &[u8]);
    fn try_write(&mut self, data: &[u8]) -> Result<(), ()>;
    fn remaining_space(&self) -> usize;
    unsafe fn filled_len_mut(&mut self) -> &mut u32;
}

pub trait ReadBuf {
    fn read(&mut self, len: usize) -> &[u8];
    fn advance(&mut self, len: usize);
    fn get_continuous(&self, len: usize) -> &[u8];
    fn remaining(&self) -> usize;
}
