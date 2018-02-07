use std::path::{Path, PathBuf};

use rocket::response::NamedFile;

#[get("/static/<file..>")]
fn assets(file: PathBuf) -> Option<NamedFile> {
    NamedFile::open(Path::new("static/").join(file)).ok()
}
