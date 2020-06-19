use crate::execution_error::ExecutionError;
use crate::execution_error::ExecutionError::OutOfGas;

#[derive(Debug)]
pub struct Memory {
    data: Vec<u8>,
}

pub const MEMORY_LIMIT: usize = 128 * 1024 * 1024;

impl Memory {
    pub fn with_capacity(capacity: usize) -> Memory {
        Memory {
            data: Vec::with_capacity(capacity),
        }
    }

    pub fn read(&mut self, offset: usize, length: usize) -> Result<&[u8], ExecutionError> {
        self.resize_if_necessary(offset + length)?;

        Ok(&self.data[offset..offset + length])
    }

    pub fn write(
        &mut self,
        offset: usize,
        length: usize,
        data: &[u8],
    ) -> Result<(), ExecutionError> {
        let data_len = data.len();
        assert!(data_len <= length);

        self.resize_if_necessary(offset + length)?;

        self.data[offset..offset + data_len].copy_from_slice(data);

        Ok(())
    }

    pub fn size(&self) -> usize {
        self.data.len()
    }

    fn resize_if_necessary(&mut self, length: usize) -> Result<(), ExecutionError> {
        let rem = length % 32;
        let rounded_length = if rem == 0 { length } else { length - rem + 32 };

        if rounded_length <= self.data.len() {
            return Ok(());
        }

        if rounded_length > MEMORY_LIMIT {
            return Err(OutOfGas);
        }

        self.data.resize(rounded_length, 0);

        Ok(())
    }
}
