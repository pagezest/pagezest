use phf::phf_map;

pub static EXTENSION_MIME_MAP: phf::Map<&'static str, &'static str> = phf_map! {
    "html" => "text/html",
    "css" => "text/css",
    "js" => "application/javascript",
    "json" => "application/json",
    "png" => "image/png",
    "jpg" => "image/jpeg",
    "gif" => "image/gif",
    "txt" => "text/plain",
    "xml" => "application/xml",
    "pdf" => "application/pdf",
};

pub fn get_mime_type(extension: &str) -> &'static str {
    EXTENSION_MIME_MAP
        .get(extension)
        .copied()
        .unwrap_or("application/octet-stream")
}
