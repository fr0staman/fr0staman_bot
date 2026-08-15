use std::io::Cursor;

use rayon::iter::{IntoParallelRefMutIterator, ParallelIterator};

/// Decode, amplify and re-encode in one hop onto the rayon pool.
pub async fn increase_sound(
    input_data: bytes::Bytes,
    volume_factor: f32,
) -> Option<Vec<u8>> {
    let (send, recv) = tokio::sync::oneshot::channel();

    rayon::spawn(move || {
        let _ = send.send(_transcode(input_data, volume_factor));
    });

    recv.await.ok()?
}

fn _transcode(input_data: bytes::Bytes, volume_factor: f32) -> Option<Vec<u8>> {
    let (mut raw, _) =
        ogg_opus::decode::<_, 48000>(Cursor::new(input_data)).ok()?;

    raw.par_iter_mut()
        .for_each(|v| *v = (*v as f32 * volume_factor) as i16);

    ogg_opus::encode::<48000, 1>(&raw).ok()
}
