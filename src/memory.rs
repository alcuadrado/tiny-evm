#[derive(Debug)]
pub struct Memory {
    data: Vec<u8>,
}

impl Memory {
    pub fn new() -> Memory {
        Memory { data: Vec::new() }
    }

    pub fn with_capacity(capacity: usize) -> Memory {
        Memory {
            data: Vec::with_capacity(capacity),
        }
    }

    pub fn read(&mut self, offset: usize, length: usize) -> &[u8] {
        self.resize_if_necessary(offset + length);

        &self.data[offset..offset + length]
    }

    pub fn write(&mut self, offset: usize, length: usize, data: &[u8]) {
        let data_len = data.len();
        assert!(data_len <= length);

        self.resize_if_necessary(offset + length);

        self.data[offset..offset + data_len].copy_from_slice(data);
    }

    pub fn size(&self) -> usize {
        self.data.len()
    }

    fn resize_if_necessary(&mut self, length: usize) {
        let rounded_length = length - length % 32 + 32;

        if rounded_length <= self.data.len() {
            return;
        }

        self.data.resize(rounded_length, 0);
    }
}
