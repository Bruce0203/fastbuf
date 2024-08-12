pub trait Buf {
    fn clear(&mut self);
    fn pos(&self) -> usize;
    fn filled_pos(&self) -> usize;
    unsafe fn set_filled_pos(&mut self, value: usize);
    unsafe fn set_pos(&mut self, value: usize);
}

pub trait WriteBuf: Buf {
    fn write(&mut self, data: &[u8]);
    fn try_write(&mut self, data: &[u8]) -> Result<(), ()>;
    fn remaining_space(&self) -> usize;
}

pub trait ReadBuf: Buf {
    fn read(&mut self, len: usize) -> &[u8];
    fn advance(&mut self, len: usize);
    fn get_continuous(&self, len: usize) -> &[u8];
    fn remaining(&self) -> usize;
}
