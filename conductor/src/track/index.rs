use super::id::SubTrackId;

#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash)]
pub enum TrackIndex {
	Main,
	Sub(SubTrackId),
}
