pub type AppResult<T> = Result<T, Box<dyn std::error::Error + Send + Sync>>;

pub fn err(message: impl Into<String>) -> Box<dyn std::error::Error + Send + Sync> {
    std::io::Error::other(message.into()).into()
}
