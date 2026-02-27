use rust_embed::RustEmbed;

#[derive(RustEmbed)]
#[folder = "../hoshi-frontend/build"]
pub struct Assets;