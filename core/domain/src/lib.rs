//! Dominio de la rockola (modelos y helpers puros).
//! Regla: Artista unificado por `artist_key` normalizado.

use unicode_normalization::UnicodeNormalization;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ArtistKey(pub String);

/// Normaliza nombre de artista para unificación:
/// - trim
/// - lowercase
/// - sin tildes (NFKD + filtrar diacríticos)
/// - remove caracteres no alfanuméricos (deja espacios)
/// - colapsa espacios
pub fn normalize_artist_key(input: &str) -> ArtistKey {
    let lower = input.trim().to_lowercase();

    let mut out = String::with_capacity(lower.len());

    // NFKD separa diacríticos; filtramos marcas combinantes.
    for ch in lower.nfkd() {
        // Filtra diacríticos (unicode combining marks)
        if is_combining_mark(ch) {
            continue;
        }
        // Permitimos letras/dígitos/espacio. Todo lo demás se vuelve espacio.
        if ch.is_alphanumeric() {
            out.push(ch);
        } else if ch.is_whitespace() {
            out.push(' ');
        } else {
            out.push(' ');
        }
    }

    // Colapsar espacios múltiples
    let collapsed = out.split_whitespace().collect::<Vec<_>>().join(" ");
    ArtistKey(collapsed)
}

fn is_combining_mark(ch: char) -> bool {
    // Rango general de marcas combinantes; suficiente para este caso.
    // (Alternativa: unicode_general_category, pero es más pesado)
    matches!(ch as u32, 0x0300..=0x036F | 0x1AB0..=0x1AFF | 0x1DC0..=0x1DFF | 0x20D0..=0x20FF | 0xFE20..=0xFE2F)
}
