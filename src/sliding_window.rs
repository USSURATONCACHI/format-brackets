
use std::io::Read;


pub struct SlidingWindow<R: Read>
{
    data: Box<[u8]>,
    start: usize,
    end: usize,
    reader: R,
    end_was_hit: bool,
}

impl<R: Read> SlidingWindow<R> {
    pub fn new(window_size: usize, reader: R) -> Self {
        let result = Self { 
            data: vec![0u8; window_size * 2 - 1].into_boxed_slice(),
            start: 0,
            end: 0,
            reader,
            end_was_hit: false,
        };

        result
    }

    fn window_size(&self) -> usize {
        (self.data.len() + 1) / 2
    }

    pub fn get_window(&self) -> &[u8] {
        return &self.data[self.start..self.end];
    }
    pub fn get_window_utf8(&self) -> &str {
        let arr = &self.get_window()[..];

        match str::from_utf8(arr) {
            Ok(valid) => valid,
            Err(err) => {
                let idx = err.valid_up_to();
                unsafe {
                    str::from_utf8_unchecked(&arr[..idx])
                }
            },
        }
    }

    fn inner_read(end_was_hit: &mut bool, reader: &mut R, mut buf: &mut [u8]) -> std::io::Result<usize> {
        let mut total_read = 0;

        while !*end_was_hit && buf.len() > 0 {
            let was_read = reader.read(buf)?;
            *end_was_hit = was_read == 0;

            total_read += was_read;
            buf = &mut buf[was_read..];
        }

        Ok(total_read)
    }

    pub fn fill(&mut self) -> std::io::Result<()> {
        let window_size = self.window_size();
        let bytes_missing = window_size - (self.end - self.start);

        if bytes_missing > 0 && !self.end_was_hit {
            self.end += Self::inner_read(&mut self.end_was_hit, &mut self.reader, &mut self.data[self.start..bytes_missing])?;
        }
        Ok(())
    }

    pub fn consume(&mut self, mut bytes_count: usize) -> std::io::Result<usize> {
        let mut buf = [0u8; 1024];
        let mut total_read = 0;

        while bytes_count > 0 {
            let read_amount = bytes_count.min(buf.len());
            total_read = self.read(&mut buf[..read_amount])?;
            bytes_count -= total_read;

            if total_read == 0 {
                break;
            }
        }

        Ok(total_read)
    }
}

impl<R: Read> Read for SlidingWindow<R> {
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        let mut total_read = 0;
        let stored_size = self.get_window().len();
        let window_size = self.window_size();

        if buf.len() >= stored_size {
            // 1. Memcopy 0..N from the window.
            buf[..stored_size].copy_from_slice(self.get_window());
            total_read += stored_size;

            // 2. Read N..end from the inner reader.
            let was_read = Self::inner_read(&mut self.end_was_hit, &mut self.reader, &mut buf[stored_size..])?;
            total_read += was_read;

            // 3. Repopulate entire window from the start
            self.start = 0;
            self.end = 0;

            let was_read = Self::inner_read(&mut self.end_was_hit, &mut self.reader, &mut self.data[..window_size])?;
            self.end += was_read;
        } else {
            // 1. Memcopy 0..end from the window
            let buf_len = buf.len();
            buf.copy_from_slice(&self.get_window()[..buf_len]);
            total_read += buf_len;
            self.start += buf_len;
            let stored_size = self.end - self.start;

            if (self.end + buf_len) <= (self.window_size() * 2 - 1) {
                let was_read = Self::inner_read(&mut self.end_was_hit, &mut self.reader, &mut self.data[self.end..(self.end + buf_len)])?;
                self.end += was_read;
            } else {
                self.data.copy_within(self.start..self.end, 0);
                self.start = 0;
                self.end = stored_size;

                let was_read = Self::inner_read(&mut self.end_was_hit, &mut self.reader, &mut self.data[self.end..window_size])?;
                self.end += was_read;
            }
        }

        Ok(total_read)
    }
}