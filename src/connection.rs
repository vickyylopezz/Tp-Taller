#[derive(PartialEq, Eq, Debug, Clone)]
pub enum InterestStatus {
    Interested,
    NotInterested,
}

#[derive(PartialEq, Eq, Debug, Clone)]
pub enum ChokeStatus {
    Choked,
    Unchoked,
}

#[derive(PartialEq, Eq, Debug, Clone)]
pub struct ConnectionStatus(pub InterestStatus, pub ChokeStatus);

impl ConnectionStatus {
    pub fn new() -> Self {
        Self(InterestStatus::NotInterested, ChokeStatus::Choked)
    }
    pub fn _request_available(&self) -> bool {
        matches!((self.0.clone(), self.1.clone()), (_, ChokeStatus::Unchoked))
    }
}
