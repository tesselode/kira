use super::id::SubTrackId;

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum TrackIndex {
	Main,
	Sub(SubTrackId),
}
