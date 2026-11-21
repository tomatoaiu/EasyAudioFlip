/// First visible character used on the tray icon.
///
/// ASCII letters are uppercased. Leading spaces and punctuation are skipped so
/// a name such as "(HDMI)" still yields a readable mark.
pub fn icon_mark(name: &str) -> Option<String> {
    let ch = name
        .chars()
        .find(|ch| !ch.is_whitespace() && !ch.is_ascii_punctuation())?;
    if ch.is_ascii_alphabetic() {
        Some(ch.to_ascii_uppercase().to_string())
    } else {
        Some(ch.to_string())
    }
}

#[cfg(windows)]
pub fn render_mark_icon(mark: &str) -> Option<tauri::image::Image<'static>> {
    let rgba = render_mark_rgba(mark)?;
    Some(tauri::image::Image::new_owned(rgba, ICON_SIZE, ICON_SIZE))
}

#[cfg(not(windows))]
pub fn render_mark_icon(_mark: &str) -> Option<tauri::image::Image<'static>> {
    None
}

#[cfg(windows)]
const ICON_SIZE: u32 = 32;

#[cfg(windows)]
fn render_mark_rgba(mark: &str) -> Option<Vec<u8>> {
    use std::mem::size_of;

    use windows::Win32::Foundation::COLORREF;
    use windows::Win32::Graphics::Gdi::{
        BI_RGB, BITMAPINFO, BITMAPINFOHEADER, CreateCompatibleDC, CreateDIBSection, DIB_RGB_COLORS,
        DeleteDC, DeleteObject, OPAQUE, SelectObject, SetBkColor, SetBkMode, SetTextColor,
    };

    let mut units: Vec<u16> = mark.encode_utf16().collect();
    if units.is_empty() {
        return None;
    }
    let glyph_len = units.len() as i32;
    units.push(0);

    let size = ICON_SIZE as i32;
    unsafe {
        let bmi = BITMAPINFO {
            bmiHeader: BITMAPINFOHEADER {
                biSize: size_of::<BITMAPINFOHEADER>() as u32,
                biWidth: size,
                biHeight: -size,
                biPlanes: 1,
                biBitCount: 32,
                biCompression: BI_RGB.0,
                ..BITMAPINFOHEADER::default()
            },
            ..BITMAPINFO::default()
        };
        let mut bits = std::ptr::null_mut();
        let bitmap = CreateDIBSection(None, &bmi, DIB_RGB_COLORS, &mut bits, None, 0).ok()?;
        let dc = CreateCompatibleDC(None);
        if dc.is_invalid() || bits.is_null() {
            let _ = DeleteObject(bitmap.into());
            if !dc.is_invalid() {
                let _ = DeleteDC(dc);
            }
            return None;
        }

        let old_bitmap = SelectObject(dc, bitmap.into());
        SetBkColor(dc, COLORREF(0));
        SetBkMode(dc, OPAQUE);
        SetTextColor(dc, COLORREF(0x00FF_FFFF));

        let font_height = -(size * 3 / 4);
        let drawn = draw_mark(dc, &units, glyph_len, font_height, size);
        let count = (size as usize) * (size as usize) * 4;
        let mask = std::slice::from_raw_parts(bits as *const u8, count).to_vec();
        SelectObject(dc, old_bitmap);
        let _ = DeleteObject(bitmap.into());
        let _ = DeleteDC(dc);
        if !drawn {
            return None;
        }

        let rgba = compose(&mask, size as usize);
        if rgba
            .chunks(4)
            .all(|px| px[3] == 0 || (px[0] == 59 && px[3] == 255))
        {
            return None;
        }
        Some(rgba)
    }
}

#[cfg(windows)]
fn draw_mark(
    dc: windows::Win32::Graphics::Gdi::HDC,
    units: &[u16],
    glyph_len: i32,
    font_height: i32,
    size: i32,
) -> bool {
    use windows::Win32::Foundation::RECT;
    use windows::Win32::Graphics::Gdi::{
        ANTIALIASED_QUALITY, CLIP_DEFAULT_PRECIS, CreateFontW, DEFAULT_CHARSET, DT_CENTER,
        DT_SINGLELINE, DT_VCENTER, DeleteObject, DrawTextW, GetGlyphIndicesW, OUT_DEFAULT_PRECIS,
        SelectObject,
    };
    use windows::core::{HSTRING, PCWSTR};

    const FACES: [&str; 4] = ["Yu Gothic UI", "Meiryo UI", "MS UI Gothic", "Segoe UI"];
    let mut text = units[..glyph_len as usize].to_vec();
    for face in FACES {
        let font = unsafe {
            CreateFontW(
                font_height,
                0,
                0,
                0,
                600,
                0,
                0,
                0,
                DEFAULT_CHARSET,
                OUT_DEFAULT_PRECIS,
                CLIP_DEFAULT_PRECIS,
                ANTIALIASED_QUALITY,
                0,
                &HSTRING::from(face),
            )
        };
        if font.is_invalid() {
            continue;
        }
        let old_font = unsafe { SelectObject(dc, font.into()) };
        let mut indices = [0u16; 2];
        let found = unsafe {
            GetGlyphIndicesW(
                dc,
                PCWSTR(units.as_ptr()),
                glyph_len,
                indices.as_mut_ptr(),
                1,
            ) != u32::MAX
                && indices[..glyph_len as usize]
                    .iter()
                    .all(|index| *index != 0xFFFF)
        };
        if found {
            let mut rect = RECT {
                left: 0,
                top: 1,
                right: size,
                bottom: size + 1,
            };
            let drew = unsafe {
                DrawTextW(
                    dc,
                    &mut text,
                    &mut rect,
                    DT_CENTER | DT_VCENTER | DT_SINGLELINE,
                )
            } > 0;
            unsafe {
                SelectObject(dc, old_font);
                let _ = DeleteObject(font.into());
            }
            return drew;
        }
        unsafe {
            SelectObject(dc, old_font);
            let _ = DeleteObject(font.into());
        }
    }
    false
}

#[cfg(windows)]
fn compose(mask_bgra: &[u8], size: usize) -> Vec<u8> {
    let radius = (size / 5).max(2);
    let mut rgba = vec![0u8; size * size * 4];
    for y in 0..size {
        for x in 0..size {
            if !in_round_rect(x, y, size, radius) {
                continue;
            }
            let i = (y * size + x) * 4;
            let coverage = mask_bgra[i].max(mask_bgra[i + 1]).max(mask_bgra[i + 2]);
            rgba[i] = blend(255, 59, coverage);
            rgba[i + 1] = blend(255, 130, coverage);
            rgba[i + 2] = blend(255, 246, coverage);
            rgba[i + 3] = 255;
        }
    }
    rgba
}

#[cfg(windows)]
fn in_round_rect(x: usize, y: usize, size: usize, radius: usize) -> bool {
    if x >= size || y >= size {
        return false;
    }
    if x >= radius && x < size - radius || y >= radius && y < size - radius {
        return true;
    }
    let cx = if x < radius {
        radius
    } else {
        size - 1 - radius
    };
    let cy = if y < radius {
        radius
    } else {
        size - 1 - radius
    };
    let dx = x.abs_diff(cx);
    let dy = y.abs_diff(cy);
    dx * dx + dy * dy <= radius * radius
}

#[cfg(windows)]
fn blend(foreground: u8, background: u8, coverage: u8) -> u8 {
    let coverage = u16::from(coverage);
    ((u16::from(foreground) * coverage + u16::from(background) * (255 - coverage)) / 255) as u8
}

#[cfg(test)]
mod tests {
    use super::icon_mark;

    #[test]
    fn takes_the_first_readable_character() {
        assert_eq!(icon_mark("Speakers (Realtek)").as_deref(), Some("S"));
        assert_eq!(icon_mark("  headphones").as_deref(), Some("H"));
        assert_eq!(icon_mark("スピーカー").as_deref(), Some("ス"));
        assert_eq!(icon_mark("(HDMI)").as_deref(), Some("H"));
        assert_eq!(icon_mark("...").as_deref(), None);
    }

    #[cfg(windows)]
    #[test]
    fn renders_a_contrasting_mark() {
        let rgba = super::render_mark_rgba("S").expect("mark icon");
        assert!(rgba.chunks(4).any(|px| px[0] > 200 && px[3] == 255));
    }
}
