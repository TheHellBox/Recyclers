pub fn serialize<T: serde::Serialize + ?Sized>(message: &T) -> Vec<u8> {
    let len = bincode::serialized_size(message).unwrap() as u32;
    let mut buf = Vec::new();
    let l = len.to_le_bytes();
    buf.extend_from_slice(&l[0..3]);
    bincode::serialize_into(&mut buf, message).unwrap();
    buf
}

pub async fn send<T: serde::Serialize>(stream: &mut quinn::SendStream, message: &T) {
    let data = serialize(message);
    if data.len() > 2u64.pow(24) as usize {
        panic!(
            "Message exceeds maximum length! ({} > {})",
            data.len(),
            2u64.pow(24)
        );
    }
    // FIXME: Return result, not unwrap
    let _ = stream.write_all(&data).await.unwrap();
}

pub async fn receive<T: serde::de::DeserializeOwned>(stream: &mut quinn::RecvStream) -> Option<T> {
    let mut l = [0; 4];
    stream.read_exact(&mut l[0..3]).await.unwrap();
    let len = u32::from_le_bytes(l) as usize;
    let mut buf = vec![0; len];
    stream.read_exact(&mut buf).await.unwrap();
    Some(bincode::deserialize(&buf).unwrap())
}
