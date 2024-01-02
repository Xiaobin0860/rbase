#[inline]
pub fn timestamp_sec() -> i64 {
    chrono::Utc::now().timestamp()
}
