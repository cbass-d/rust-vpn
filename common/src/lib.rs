pub mod errors;
pub mod messages;

static NOISE_PATTERN: &'static str = "Noise_IK_22519_ChaChaPoly_BLAKE2s";

pub enum PeerState {
    Handshaking,
    Connected,
    Closed,
}
