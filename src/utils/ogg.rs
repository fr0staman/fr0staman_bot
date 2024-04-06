use std::io::Cursor;

use rayon::iter::{IntoParallelRefMutIterator, ParallelIterator};

pub async fn increase_sound(
    input_data: bytes::Bytes,
    volume_factor: f32,
) -> Option<Vec<u8>> {
    let raw = _decode(input_data).await?;

    let raw = _increase(raw, volume_factor).await?;

    _encode(raw).await
}

async fn _decode(input_data: bytes::Bytes) -> Option<Vec<i16>> {
    let (send, recv) = tokio::sync::oneshot::channel();

    rayon::spawn(move || {
        let decode_res =
            ogg_opus::decode::<_, 48000>(Cursor::new(input_data)).ok();

        let _ = send.send(decode_res.map(|v| v.0));
    });

    recv.await.ok()?
}

async fn _increase(mut raw: Vec<i16>, volume_factor: f32) -> Option<Vec<i16>> {
    let (send, recv) = tokio::sync::oneshot::channel();

    rayon::spawn(move || {
        raw.par_iter_mut()
            .for_each(|v| *v = (*v as f32 * volume_factor) as i16);

        let _ = send.send(raw);
    });

    recv.await.ok()
}

async fn _encode(raw: Vec<i16>) -> Option<Vec<u8>> {
    let (send, recv) = tokio::sync::oneshot::channel();

    rayon::spawn(move || {
        let encoded = ogg_opus::encode::<48000, 1>(&raw).ok();
        let _ = send.send(encoded);
    });

    recv.await.ok()?
}
