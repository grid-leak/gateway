pub mod auth;
pub mod pamplona;
pub mod pamplona_authenticated;

pub fn map_err<E: std::fmt::Display>(e: E) -> jsonrpsee::types::ErrorObjectOwned {
    jsonrpsee::types::ErrorObject::owned(
        jsonrpsee::types::error::INTERNAL_ERROR_CODE,
        e.to_string(),
        None::<()>,
    )
}