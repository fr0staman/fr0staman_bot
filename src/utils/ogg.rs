use std::io::Cursor;

pub async fn increase_sound(
    input_data: bytes::Bytes,
    volume_factor: f32,
) -> Option<Vec<u8>> {
    let mut raw = tokio::task::spawn_blocking(move || {
        let (raw, _header) =
            ogg_opus::decode::<_, 48000>(Cursor::new(input_data)).ok()?;
        Some(raw)
    })
    .await
    .ok()??;

    let raw = tokio::task::spawn_blocking(move || {
        for num in &mut raw {
            *num = (*num as f32 * volume_factor) as i16;
        }
        raw
    })
    .await
    .ok()?;

    tokio::task::spawn_blocking(move || ogg_opus::encode::<48000, 1>(&raw).ok())
        .await
        .ok()?
}
