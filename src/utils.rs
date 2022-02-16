//! general utilities that help to process audio data

/// seperates 1 dimensional interleaved audio stream to 2D vector of audiodata of each channel
pub fn seperate_channels(data: &[f32], channel_count: usize) -> Vec<Vec<f32>> {
    let mut buffer: Vec<Vec<f32>> = vec![vec![]; channel_count];

    for chunked_data in data.chunks(channel_count) {
        for (i, v) in chunked_data.iter().enumerate() {
            buffer[i].push(*v);
        }
    }

    buffer
}