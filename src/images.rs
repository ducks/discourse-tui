use regex::Regex;

pub fn extract_image_urls(html: &str) -> Vec<String> {
    let mut urls = Vec::new();

    // Match img tags with src attribute
    let re = Regex::new(r#"<img[^>]+src=["']([^"']+)["']"#).unwrap();

    for cap in re.captures_iter(html) {
        if let Some(url) = cap.get(1) {
            urls.push(url.as_str().to_string());
        }
    }

    urls
}

// TODO: Implement image downloading and rendering
// pub fn download_image(url: &str) -> Result<DynamicImage, Box<dyn std::error::Error>>
// pub fn resize_image_for_terminal(img: DynamicImage, max_width: u32) -> DynamicImage
