use image::{
    imageops::{overlay, FilterType},
    load_from_memory, DynamicImage, Rgba,
};
use imageproc::drawing::draw_text_mut as imageproc_draw_text_mut;
#[cfg(all(feature = "discord", any(feature = "async-std", feature = "tokio")))]
use isahc::AsyncReadResponseExt;
#[cfg(all(
    feature = "discord",
    not(feature = "async-std"),
    not(feature = "tokio")
))]
use isahc::ReadResponseExt;
use rusttype::{point, Font, Scale};
use rustwemoji_parser::{parse, Token};

pub fn measure_line(font: &Font, text: &str, scale: Scale) -> (f32, f32) {
    let width = font
        .layout(text, scale, point(0.0, 0.0))
        .map(|g| g.position().x + g.unpositioned().h_metrics().advance_width)
        .last()
        .unwrap_or(0.0);

    let v_metrics = font.v_metrics(scale);
    let height = v_metrics.ascent - v_metrics.descent + v_metrics.line_gap;

    (width, height)
}

#[cfg(any(feature = "async-std", feature = "tokio"))]
pub async fn draw_tokens_mut(
    img: &mut DynamicImage,
    tokens: Vec<Token>,
    x: i64,
    y: i64,
    font: &Font<'_>,
    color: Rgba<u8>,
    scale: Scale,
) {
    let mut x = x;
    let fontsize = scale.y as u32;
    for i in tokens {
        match i {
            Token::Text(t) => {
                imageproc_draw_text_mut(
                    img,
                    color,
                    x.clone().try_into().unwrap(),
                    y.clone().try_into().unwrap(),
                    scale,
                    font,
                    &t,
                );
                let (width, _) = measure_line(font, &t, scale);
                x = x + width as i64;
            }
            Token::Emoji(e) => {
                if let Ok(mut k) = load_from_memory(e.as_slice()) {
                    k = k.resize(fontsize, fontsize, FilterType::Triangle);
                    overlay(img, &k, x, y);
                    x = x + i64::from(fontsize);
                }
            }
            #[cfg(feature = "discord")]
            Token::CustomEmoji(e) =>
            {
                #[cfg(any(feature = "async-std", feature = "tokio"))]
                if let Ok(mut r) = isahc::get_async(&e).await {
                    if let Ok(b) = r.bytes().await {
                        load_from_memory(&b).ok().map(|mut k| {
                            k = k.resize(fontsize, fontsize, FilterType::Triangle);
                            overlay(img, &k, x, y);
                            x = x + i64::from(fontsize);
                        });
                    }
                }
            }
        }
    }
}

#[cfg(all(not(feature = "async-std"), not(feature = "tokio")))]
fn draw_tokens_mut(
    img: &mut DynamicImage,
    tokens: Vec<Token>,
    x: i64,
    y: i64,
    font: &Font<'_>,
    color: Rgba<u8>,
    scale: Scale,
) {
    let mut x = x;
    let fontsize = scale.y as u32;
    for i in tokens {
        match i {
            Token::Text(t) => {
                imageproc_draw_text_mut(
                    img,
                    color,
                    x.clone().try_into().unwrap(),
                    y.clone().try_into().unwrap(),
                    scale,
                    font,
                    &t,
                );
                let (width, _) = measure_line(font, &t, scale);
                x = x + width as i64;
            }
            Token::Emoji(e) => {
                if let Ok(mut k) = load_from_memory(e.as_slice()) {
                    k = k.resize(fontsize, fontsize, FilterType::Triangle);
                    overlay(img, &k, x, y);
                    x = x + i64::from(fontsize);
                }
            }
            #[cfg(feature = "discord")]
            Token::CustomEmoji(e) => {
                isahc::get(&e)
                    .ok()
                    .and_then(|mut r| r.bytes().ok())
                    .and_then(|b| load_from_memory(&b).ok())
                    .map(|mut k| {
                        k = k.resize(fontsize, fontsize, FilterType::Triangle);
                        overlay(img, &k, x, y);
                        x = x + i64::from(fontsize);
                    });
            }
        }
    }
}

#[cfg(all(not(feature = "async-std"), not(feature = "tokio")))]
pub fn draw_text_mut(
    img: &mut DynamicImage,
    text: String,
    x: i64,
    y: i64,
    font: &Font,
    color: Rgba<u8>,
    scale: Scale,
) {
    let tokens = parse(text);
    draw_tokens_mut(img, tokens, x, y, font, color, scale);
}

#[cfg(feature = "tokio")]
pub async fn draw_text_mut(
    img: &mut DynamicImage,
    text: String,
    x: i64,
    y: i64,
    font: &Font<'_>,
    color: Rgba<u8>,
    scale: Scale,
) {
    if let Ok(tokens) = parse(text).await {
        draw_tokens_mut(img, tokens, x, y, font, color, scale).await;
    }
}

#[cfg(feature = "async-std")]
pub async fn draw_text_mut(
    img: &mut DynamicImage,
    text: String,
    x: i64,
    y: i64,
    font: &Font<'_>,
    color: Rgba<u8>,
    scale: Scale,
) {
    let tokens = parse(text).await;
    draw_tokens_mut(img, tokens, x, y, font, color, scale).await;
}

#[cfg(test)]
mod test {
    use super::draw_text_mut;

    #[cfg(feature = "async-std")]
    #[async_std::test]
    async fn test_draw_text_mut() {
        use image::DynamicImage;
        let mut img = DynamicImage::new_rgb8(100, 100);
        let font = super::Font::try_from_bytes(include_bytes!("../test/NotoSansJP-Regular.ttf"))
            .expect("Failed to load font");
        let text = "Hello World!".to_string();
        draw_text_mut(
            &mut img,
            text,
            0,
            0,
            &font,
            image::Rgba([0, 0, 0, 255]),
            super::Scale::uniform(12.0),
        )
        .await;
    }

    #[cfg(feature = "tokio")]
    #[tokio::test]
    async fn test_draw_text_mut() {
        use image::DynamicImage;
        let mut img = DynamicImage::new_rgb8(100, 100);
        let font = super::Font::try_from_bytes(include_bytes!("../test/NotoSansJP-Regular.ttf"))
            .expect("Failed to load font");
        let text = "Hello World!".to_string();
        draw_text_mut(
            &mut img,
            text,
            0,
            0,
            &font,
            image::Rgba([0, 0, 0, 255]),
            super::Scale::uniform(12.0),
        )
        .await;
    }

    #[cfg(all(not(feature = "async-std"), not(feature = "tokio")))]
    #[test]
    fn test_draw_text_mut() {
        use image::DynamicImage;
        let mut img = DynamicImage::new_rgb8(100, 100);
        let font = super::Font::try_from_bytes(include_bytes!("../test/NotoSansJP-Regular.ttf"))
            .expect("Failed to load font");
        let text = "Hello World!".to_string();
        draw_text_mut(
            &mut img,
            text,
            0,
            0,
            &font,
            image::Rgba([0, 0, 0, 255]),
            super::Scale::uniform(12.0),
        );
    }
}
