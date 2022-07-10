pub fn i16_to_f32(sample: &[i16]) -> Vec<f32> {
    let f32_sample: Vec<f32> = sample
        .into_iter()
        .map(|x| *x as f32 / i16::MAX as f32)
        .collect();

    f32_sample
}

pub fn u16_to_f32(sample: &[u16]) -> Vec<f32> {
    let f32_sample: Vec<f32> = sample
        .into_iter()
        .map(|x| *x as f32 / u16::MAX as f32 - 0.5)
        .collect();

    f32_sample
}