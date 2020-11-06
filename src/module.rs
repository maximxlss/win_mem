use winapi::shared::minwindef::DWORD;

pub struct Module {
    pub(crate) name: String,
    /// Memory address of the [`Module`] relative to the process
    pub(crate) address: DWORD,
    /// Length of the [`Module`] in bytes
    pub(crate) len: DWORD,
}

impl Module {
    #[inline]
    pub fn name(&self) -> &str {
        &self.name
    }
    #[inline]
    pub fn address(&self) -> DWORD {
        self.address
    }
    #[inline]
    pub fn len(&self) -> DWORD {
        self.len
    }
}