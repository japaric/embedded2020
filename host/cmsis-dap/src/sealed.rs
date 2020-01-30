/// Data that can be read from memory using the AHB-AP (Access Port)
pub trait Data {
    const BYTES: u16;
    const CSW_SIZE: u32;

    fn acronym() -> &'static str;
    fn push(memory: &mut Vec<Self>, bytes: &[u8], offset: u32, count: u16)
    where
        Self: Sized;
}
