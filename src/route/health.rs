use rocket::State;
use rocket::http::Status;
use rocket_slog::SyncLogger;

use crate::config::Config;
use crate::response::Response;

/// Returns just OK status. This route should be mounted both endpoints.
#[get("/health", rank = 1)]
pub fn check<'a>(logger: SyncLogger, _state: State<Config>) -> Response<'a> {
    info!(logger, "");
    let res: Response = Default::default();
    res.status(Status::Ok)
}
