use askama_axum::Template;

#[derive(Template)]
#[template(path = "login.html")]
pub struct LoginTemplate;

#[derive(Template)]
#[template(path = "upload.html")]
pub struct UploadTemplate;

#[derive(Template)]
#[template(path = "admin.html")]
pub struct AdminTemplate;

#[derive(Template)]
#[template(path = "logs.html")]
pub struct LogsTemplate;

#[derive(Template)]
#[template(path = "settings.html")]
pub struct SettingsTemplate;
