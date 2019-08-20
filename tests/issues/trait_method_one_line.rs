pub trait Request: Message {
    fn is_response(&self) -> bool { false }
}
