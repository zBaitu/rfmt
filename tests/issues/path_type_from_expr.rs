pub trait Method: Encodable + Decodable {
    fn se(&self) -> AmqpResult<Vec<u8>> {
        let mut encoder = Encoder::<Self>::new();
        Ok(encoder.data)
    }
}

