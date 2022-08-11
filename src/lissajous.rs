use crate::utils::seperate_channels;

/// adds delay between two audio channels to improve the lissajous figure
/// this introduces loss of information
/// 
/// channel count is arbitrary, it will always output 2 channels
pub struct StereoDelayer {
    l_buffer: Vec<f32>,
    r_buffer: Vec<f32>,
    channel_count: usize,
    overlap: f32,
    offset: f32,
    buffer_cap: usize,
}
impl StereoDelayer {
    pub fn new(channel_count: usize, overlap: f32, offset: f32, buffer_cap: usize) -> Self {
        Self {
            l_buffer: Vec::new(),
            r_buffer: Vec::new(),
            channel_count,
            overlap,
            offset,
            buffer_cap,
        }
    }

    pub fn update(&mut self) -> Vec<[f32; 2]> {
        assert_eq!(self.l_buffer.len(), self.r_buffer.len());

        // clear buffer
        let end: usize = self.l_buffer.len() - (self.l_buffer.len() as f32 * self.overlap) as usize;
        self.l_buffer.drain(0..end);
        self.r_buffer.drain(0..end);

        let mut out = [
            self.l_buffer.clone(),
            self.r_buffer.clone(),
        ];

        // apply offset
        let offset = (self.offset * out[0].len() as f32).floor() as usize;
        for _ in 0..offset {
            if !out[0].is_empty() {
                out[0].remove(0);
                out[1].pop();
            }
        }

        let mut combined: Vec<[f32; 2]> = Vec::new();
        for i in 0..out[0].len() {
            combined.push([
                out[0][i],
                out[1][i],
            ]);
        }

        combined
    }

    pub fn push_data(&mut self, data: &[f32]) {
        let out = if self.channel_count >= 2 {
            seperate_channels(&data, self.channel_count)
        } else {
            vec![
                data.to_vec(),
                data.to_vec(),
            ]
        };
        self.l_buffer.append(&mut out[0].clone());
        self.r_buffer.append(&mut out[1].clone());

        // cap
        if self.l_buffer.len() > self.buffer_cap * self.channel_count {
            let end = self.l_buffer.len() - self.buffer_cap;
            self.l_buffer.drain(0..end);
            self.r_buffer.drain(0..end);
        }
    }
}