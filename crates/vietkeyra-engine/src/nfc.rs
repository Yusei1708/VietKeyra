//! Pure Rust Unicode NFC Lookup Table for Vietnamese Vowels and Consonants
//!
//! Provides O(1) constant-time lookup for all 144 Vietnamese vowel-modifier-tone-casing
//! combinations and D-stroke consonants without any heap allocation or external dependencies.

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum BaseVowel {
    A,
    E,
    I,
    O,
    U,
    Y,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Modifier {
    None,
    Circumflex, // â, ê, ô
    Breve,      // ă
    Horn,       // ơ, ư
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum Tone {
    #[default]
    None = 0,
    Sac = 1,   // Sắc (acute, 's')
    Huyen = 2, // Huyền (grave, 'f')
    Hoi = 3,   // Hỏi (hook, 'r')
    Nga = 4,   // Ngã (tilde, 'x')
    Nang = 5,  // Nặng (dot below, 'j')
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Case {
    Lower,
    Upper,
}

/// Composes a base vowel, modifier, tone, and casing into a single Unicode NFC character.
/// Returns None if the combination is invalid (e.g. Breve on E).
#[inline]
pub fn compose_vowel(base: BaseVowel, modifier: Modifier, tone: Tone, case: Case) -> Option<char> {
    let ch = match (base, modifier, tone, case) {
        (BaseVowel::A, Modifier::None, Tone::None, Case::Lower) => 'a',
        (BaseVowel::A, Modifier::None, Tone::None, Case::Upper) => 'A',
        (BaseVowel::A, Modifier::None, Tone::Sac, Case::Lower) => 'á',
        (BaseVowel::A, Modifier::None, Tone::Sac, Case::Upper) => 'Á',
        (BaseVowel::A, Modifier::None, Tone::Huyen, Case::Lower) => 'à',
        (BaseVowel::A, Modifier::None, Tone::Huyen, Case::Upper) => 'À',
        (BaseVowel::A, Modifier::None, Tone::Hoi, Case::Lower) => 'ả',
        (BaseVowel::A, Modifier::None, Tone::Hoi, Case::Upper) => 'Ả',
        (BaseVowel::A, Modifier::None, Tone::Nga, Case::Lower) => 'ã',
        (BaseVowel::A, Modifier::None, Tone::Nga, Case::Upper) => 'Ã',
        (BaseVowel::A, Modifier::None, Tone::Nang, Case::Lower) => 'ạ',
        (BaseVowel::A, Modifier::None, Tone::Nang, Case::Upper) => 'Ạ',
        (BaseVowel::A, Modifier::Circumflex, Tone::None, Case::Lower) => 'â',
        (BaseVowel::A, Modifier::Circumflex, Tone::None, Case::Upper) => 'Â',
        (BaseVowel::A, Modifier::Circumflex, Tone::Sac, Case::Lower) => 'ấ',
        (BaseVowel::A, Modifier::Circumflex, Tone::Sac, Case::Upper) => 'Ấ',
        (BaseVowel::A, Modifier::Circumflex, Tone::Huyen, Case::Lower) => 'ầ',
        (BaseVowel::A, Modifier::Circumflex, Tone::Huyen, Case::Upper) => 'Ầ',
        (BaseVowel::A, Modifier::Circumflex, Tone::Hoi, Case::Lower) => 'ẩ',
        (BaseVowel::A, Modifier::Circumflex, Tone::Hoi, Case::Upper) => 'Ẩ',
        (BaseVowel::A, Modifier::Circumflex, Tone::Nga, Case::Lower) => 'ẫ',
        (BaseVowel::A, Modifier::Circumflex, Tone::Nga, Case::Upper) => 'Ẫ',
        (BaseVowel::A, Modifier::Circumflex, Tone::Nang, Case::Lower) => 'ậ',
        (BaseVowel::A, Modifier::Circumflex, Tone::Nang, Case::Upper) => 'Ậ',
        (BaseVowel::A, Modifier::Breve, Tone::None, Case::Lower) => 'ă',
        (BaseVowel::A, Modifier::Breve, Tone::None, Case::Upper) => 'Ă',
        (BaseVowel::A, Modifier::Breve, Tone::Sac, Case::Lower) => 'ắ',
        (BaseVowel::A, Modifier::Breve, Tone::Sac, Case::Upper) => 'Ắ',
        (BaseVowel::A, Modifier::Breve, Tone::Huyen, Case::Lower) => 'ằ',
        (BaseVowel::A, Modifier::Breve, Tone::Huyen, Case::Upper) => 'Ằ',
        (BaseVowel::A, Modifier::Breve, Tone::Hoi, Case::Lower) => 'ẳ',
        (BaseVowel::A, Modifier::Breve, Tone::Hoi, Case::Upper) => 'Ẳ',
        (BaseVowel::A, Modifier::Breve, Tone::Nga, Case::Lower) => 'ẵ',
        (BaseVowel::A, Modifier::Breve, Tone::Nga, Case::Upper) => 'Ẵ',
        (BaseVowel::A, Modifier::Breve, Tone::Nang, Case::Lower) => 'ặ',
        (BaseVowel::A, Modifier::Breve, Tone::Nang, Case::Upper) => 'Ặ',
        (BaseVowel::E, Modifier::None, Tone::None, Case::Lower) => 'e',
        (BaseVowel::E, Modifier::None, Tone::None, Case::Upper) => 'E',
        (BaseVowel::E, Modifier::None, Tone::Sac, Case::Lower) => 'é',
        (BaseVowel::E, Modifier::None, Tone::Sac, Case::Upper) => 'É',
        (BaseVowel::E, Modifier::None, Tone::Huyen, Case::Lower) => 'è',
        (BaseVowel::E, Modifier::None, Tone::Huyen, Case::Upper) => 'È',
        (BaseVowel::E, Modifier::None, Tone::Hoi, Case::Lower) => 'ẻ',
        (BaseVowel::E, Modifier::None, Tone::Hoi, Case::Upper) => 'Ẻ',
        (BaseVowel::E, Modifier::None, Tone::Nga, Case::Lower) => 'ẽ',
        (BaseVowel::E, Modifier::None, Tone::Nga, Case::Upper) => 'Ẽ',
        (BaseVowel::E, Modifier::None, Tone::Nang, Case::Lower) => 'ẹ',
        (BaseVowel::E, Modifier::None, Tone::Nang, Case::Upper) => 'Ẹ',
        (BaseVowel::E, Modifier::Circumflex, Tone::None, Case::Lower) => 'ê',
        (BaseVowel::E, Modifier::Circumflex, Tone::None, Case::Upper) => 'Ê',
        (BaseVowel::E, Modifier::Circumflex, Tone::Sac, Case::Lower) => 'ế',
        (BaseVowel::E, Modifier::Circumflex, Tone::Sac, Case::Upper) => 'Ế',
        (BaseVowel::E, Modifier::Circumflex, Tone::Huyen, Case::Lower) => 'ề',
        (BaseVowel::E, Modifier::Circumflex, Tone::Huyen, Case::Upper) => 'Ề',
        (BaseVowel::E, Modifier::Circumflex, Tone::Hoi, Case::Lower) => 'ể',
        (BaseVowel::E, Modifier::Circumflex, Tone::Hoi, Case::Upper) => 'Ể',
        (BaseVowel::E, Modifier::Circumflex, Tone::Nga, Case::Lower) => 'ễ',
        (BaseVowel::E, Modifier::Circumflex, Tone::Nga, Case::Upper) => 'Ễ',
        (BaseVowel::E, Modifier::Circumflex, Tone::Nang, Case::Lower) => 'ệ',
        (BaseVowel::E, Modifier::Circumflex, Tone::Nang, Case::Upper) => 'Ệ',
        (BaseVowel::I, Modifier::None, Tone::None, Case::Lower) => 'i',
        (BaseVowel::I, Modifier::None, Tone::None, Case::Upper) => 'I',
        (BaseVowel::I, Modifier::None, Tone::Sac, Case::Lower) => 'í',
        (BaseVowel::I, Modifier::None, Tone::Sac, Case::Upper) => 'Í',
        (BaseVowel::I, Modifier::None, Tone::Huyen, Case::Lower) => 'ì',
        (BaseVowel::I, Modifier::None, Tone::Huyen, Case::Upper) => 'Ì',
        (BaseVowel::I, Modifier::None, Tone::Hoi, Case::Lower) => 'ỉ',
        (BaseVowel::I, Modifier::None, Tone::Hoi, Case::Upper) => 'Ỉ',
        (BaseVowel::I, Modifier::None, Tone::Nga, Case::Lower) => 'ĩ',
        (BaseVowel::I, Modifier::None, Tone::Nga, Case::Upper) => 'Ĩ',
        (BaseVowel::I, Modifier::None, Tone::Nang, Case::Lower) => 'ị',
        (BaseVowel::I, Modifier::None, Tone::Nang, Case::Upper) => 'Ị',
        (BaseVowel::O, Modifier::None, Tone::None, Case::Lower) => 'o',
        (BaseVowel::O, Modifier::None, Tone::None, Case::Upper) => 'O',
        (BaseVowel::O, Modifier::None, Tone::Sac, Case::Lower) => 'ó',
        (BaseVowel::O, Modifier::None, Tone::Sac, Case::Upper) => 'Ó',
        (BaseVowel::O, Modifier::None, Tone::Huyen, Case::Lower) => 'ò',
        (BaseVowel::O, Modifier::None, Tone::Huyen, Case::Upper) => 'Ò',
        (BaseVowel::O, Modifier::None, Tone::Hoi, Case::Lower) => 'ỏ',
        (BaseVowel::O, Modifier::None, Tone::Hoi, Case::Upper) => 'Ỏ',
        (BaseVowel::O, Modifier::None, Tone::Nga, Case::Lower) => 'õ',
        (BaseVowel::O, Modifier::None, Tone::Nga, Case::Upper) => 'Õ',
        (BaseVowel::O, Modifier::None, Tone::Nang, Case::Lower) => 'ọ',
        (BaseVowel::O, Modifier::None, Tone::Nang, Case::Upper) => 'Ọ',
        (BaseVowel::O, Modifier::Circumflex, Tone::None, Case::Lower) => 'ô',
        (BaseVowel::O, Modifier::Circumflex, Tone::None, Case::Upper) => 'Ô',
        (BaseVowel::O, Modifier::Circumflex, Tone::Sac, Case::Lower) => 'ố',
        (BaseVowel::O, Modifier::Circumflex, Tone::Sac, Case::Upper) => 'Ố',
        (BaseVowel::O, Modifier::Circumflex, Tone::Huyen, Case::Lower) => 'ồ',
        (BaseVowel::O, Modifier::Circumflex, Tone::Huyen, Case::Upper) => 'Ồ',
        (BaseVowel::O, Modifier::Circumflex, Tone::Hoi, Case::Lower) => 'ổ',
        (BaseVowel::O, Modifier::Circumflex, Tone::Hoi, Case::Upper) => 'Ổ',
        (BaseVowel::O, Modifier::Circumflex, Tone::Nga, Case::Lower) => 'ỗ',
        (BaseVowel::O, Modifier::Circumflex, Tone::Nga, Case::Upper) => 'Ỗ',
        (BaseVowel::O, Modifier::Circumflex, Tone::Nang, Case::Lower) => 'ộ',
        (BaseVowel::O, Modifier::Circumflex, Tone::Nang, Case::Upper) => 'Ộ',
        (BaseVowel::O, Modifier::Horn, Tone::None, Case::Lower) => 'ơ',
        (BaseVowel::O, Modifier::Horn, Tone::None, Case::Upper) => 'Ơ',
        (BaseVowel::O, Modifier::Horn, Tone::Sac, Case::Lower) => 'ớ',
        (BaseVowel::O, Modifier::Horn, Tone::Sac, Case::Upper) => 'Ớ',
        (BaseVowel::O, Modifier::Horn, Tone::Huyen, Case::Lower) => 'ờ',
        (BaseVowel::O, Modifier::Horn, Tone::Huyen, Case::Upper) => 'Ờ',
        (BaseVowel::O, Modifier::Horn, Tone::Hoi, Case::Lower) => 'ở',
        (BaseVowel::O, Modifier::Horn, Tone::Hoi, Case::Upper) => 'Ở',
        (BaseVowel::O, Modifier::Horn, Tone::Nga, Case::Lower) => 'ỡ',
        (BaseVowel::O, Modifier::Horn, Tone::Nga, Case::Upper) => 'Ỡ',
        (BaseVowel::O, Modifier::Horn, Tone::Nang, Case::Lower) => 'ợ',
        (BaseVowel::O, Modifier::Horn, Tone::Nang, Case::Upper) => 'Ợ',
        (BaseVowel::U, Modifier::None, Tone::None, Case::Lower) => 'u',
        (BaseVowel::U, Modifier::None, Tone::None, Case::Upper) => 'U',
        (BaseVowel::U, Modifier::None, Tone::Sac, Case::Lower) => 'ú',
        (BaseVowel::U, Modifier::None, Tone::Sac, Case::Upper) => 'Ú',
        (BaseVowel::U, Modifier::None, Tone::Huyen, Case::Lower) => 'ù',
        (BaseVowel::U, Modifier::None, Tone::Huyen, Case::Upper) => 'Ù',
        (BaseVowel::U, Modifier::None, Tone::Hoi, Case::Lower) => 'ủ',
        (BaseVowel::U, Modifier::None, Tone::Hoi, Case::Upper) => 'Ủ',
        (BaseVowel::U, Modifier::None, Tone::Nga, Case::Lower) => 'ũ',
        (BaseVowel::U, Modifier::None, Tone::Nga, Case::Upper) => 'Ũ',
        (BaseVowel::U, Modifier::None, Tone::Nang, Case::Lower) => 'ụ',
        (BaseVowel::U, Modifier::None, Tone::Nang, Case::Upper) => 'Ụ',
        (BaseVowel::U, Modifier::Horn, Tone::None, Case::Lower) => 'ư',
        (BaseVowel::U, Modifier::Horn, Tone::None, Case::Upper) => 'Ư',
        (BaseVowel::U, Modifier::Horn, Tone::Sac, Case::Lower) => 'ứ',
        (BaseVowel::U, Modifier::Horn, Tone::Sac, Case::Upper) => 'Ứ',
        (BaseVowel::U, Modifier::Horn, Tone::Huyen, Case::Lower) => 'ừ',
        (BaseVowel::U, Modifier::Horn, Tone::Huyen, Case::Upper) => 'Ừ',
        (BaseVowel::U, Modifier::Horn, Tone::Hoi, Case::Lower) => 'ử',
        (BaseVowel::U, Modifier::Horn, Tone::Hoi, Case::Upper) => 'Ử',
        (BaseVowel::U, Modifier::Horn, Tone::Nga, Case::Lower) => 'ữ',
        (BaseVowel::U, Modifier::Horn, Tone::Nga, Case::Upper) => 'Ữ',
        (BaseVowel::U, Modifier::Horn, Tone::Nang, Case::Lower) => 'ự',
        (BaseVowel::U, Modifier::Horn, Tone::Nang, Case::Upper) => 'Ự',
        (BaseVowel::Y, Modifier::None, Tone::None, Case::Lower) => 'y',
        (BaseVowel::Y, Modifier::None, Tone::None, Case::Upper) => 'Y',
        (BaseVowel::Y, Modifier::None, Tone::Sac, Case::Lower) => 'ý',
        (BaseVowel::Y, Modifier::None, Tone::Sac, Case::Upper) => 'Ý',
        (BaseVowel::Y, Modifier::None, Tone::Huyen, Case::Lower) => 'ỳ',
        (BaseVowel::Y, Modifier::None, Tone::Huyen, Case::Upper) => 'Ỳ',
        (BaseVowel::Y, Modifier::None, Tone::Hoi, Case::Lower) => 'ỷ',
        (BaseVowel::Y, Modifier::None, Tone::Hoi, Case::Upper) => 'Ỷ',
        (BaseVowel::Y, Modifier::None, Tone::Nga, Case::Lower) => 'ỹ',
        (BaseVowel::Y, Modifier::None, Tone::Nga, Case::Upper) => 'Ỹ',
        (BaseVowel::Y, Modifier::None, Tone::Nang, Case::Lower) => 'ỵ',
        (BaseVowel::Y, Modifier::None, Tone::Nang, Case::Upper) => 'Ỵ',
        _ => return None,
    };
    Some(ch)
}

/// Decomposes any Vietnamese vowel NFC character into its constituent parts.
/// Returns None if the character is not a recognized Vietnamese vowel.
#[inline]
pub fn decompose_vowel(ch: char) -> Option<(BaseVowel, Modifier, Tone, Case)> {
    match ch {
        'a' => Some((BaseVowel::A, Modifier::None, Tone::None, Case::Lower)),
        'A' => Some((BaseVowel::A, Modifier::None, Tone::None, Case::Upper)),
        'á' => Some((BaseVowel::A, Modifier::None, Tone::Sac, Case::Lower)),
        'Á' => Some((BaseVowel::A, Modifier::None, Tone::Sac, Case::Upper)),
        'à' => Some((BaseVowel::A, Modifier::None, Tone::Huyen, Case::Lower)),
        'À' => Some((BaseVowel::A, Modifier::None, Tone::Huyen, Case::Upper)),
        'ả' => Some((BaseVowel::A, Modifier::None, Tone::Hoi, Case::Lower)),
        'Ả' => Some((BaseVowel::A, Modifier::None, Tone::Hoi, Case::Upper)),
        'ã' => Some((BaseVowel::A, Modifier::None, Tone::Nga, Case::Lower)),
        'Ã' => Some((BaseVowel::A, Modifier::None, Tone::Nga, Case::Upper)),
        'ạ' => Some((BaseVowel::A, Modifier::None, Tone::Nang, Case::Lower)),
        'Ạ' => Some((BaseVowel::A, Modifier::None, Tone::Nang, Case::Upper)),
        'â' => Some((BaseVowel::A, Modifier::Circumflex, Tone::None, Case::Lower)),
        'Â' => Some((BaseVowel::A, Modifier::Circumflex, Tone::None, Case::Upper)),
        'ấ' => Some((BaseVowel::A, Modifier::Circumflex, Tone::Sac, Case::Lower)),
        'Ấ' => Some((BaseVowel::A, Modifier::Circumflex, Tone::Sac, Case::Upper)),
        'ầ' => Some((BaseVowel::A, Modifier::Circumflex, Tone::Huyen, Case::Lower)),
        'Ầ' => Some((BaseVowel::A, Modifier::Circumflex, Tone::Huyen, Case::Upper)),
        'ẩ' => Some((BaseVowel::A, Modifier::Circumflex, Tone::Hoi, Case::Lower)),
        'Ẩ' => Some((BaseVowel::A, Modifier::Circumflex, Tone::Hoi, Case::Upper)),
        'ẫ' => Some((BaseVowel::A, Modifier::Circumflex, Tone::Nga, Case::Lower)),
        'Ẫ' => Some((BaseVowel::A, Modifier::Circumflex, Tone::Nga, Case::Upper)),
        'ậ' => Some((BaseVowel::A, Modifier::Circumflex, Tone::Nang, Case::Lower)),
        'Ậ' => Some((BaseVowel::A, Modifier::Circumflex, Tone::Nang, Case::Upper)),
        'ă' => Some((BaseVowel::A, Modifier::Breve, Tone::None, Case::Lower)),
        'Ă' => Some((BaseVowel::A, Modifier::Breve, Tone::None, Case::Upper)),
        'ắ' => Some((BaseVowel::A, Modifier::Breve, Tone::Sac, Case::Lower)),
        'Ắ' => Some((BaseVowel::A, Modifier::Breve, Tone::Sac, Case::Upper)),
        'ằ' => Some((BaseVowel::A, Modifier::Breve, Tone::Huyen, Case::Lower)),
        'Ằ' => Some((BaseVowel::A, Modifier::Breve, Tone::Huyen, Case::Upper)),
        'ẳ' => Some((BaseVowel::A, Modifier::Breve, Tone::Hoi, Case::Lower)),
        'Ẳ' => Some((BaseVowel::A, Modifier::Breve, Tone::Hoi, Case::Upper)),
        'ẵ' => Some((BaseVowel::A, Modifier::Breve, Tone::Nga, Case::Lower)),
        'Ẵ' => Some((BaseVowel::A, Modifier::Breve, Tone::Nga, Case::Upper)),
        'ặ' => Some((BaseVowel::A, Modifier::Breve, Tone::Nang, Case::Lower)),
        'Ặ' => Some((BaseVowel::A, Modifier::Breve, Tone::Nang, Case::Upper)),
        'e' => Some((BaseVowel::E, Modifier::None, Tone::None, Case::Lower)),
        'E' => Some((BaseVowel::E, Modifier::None, Tone::None, Case::Upper)),
        'é' => Some((BaseVowel::E, Modifier::None, Tone::Sac, Case::Lower)),
        'É' => Some((BaseVowel::E, Modifier::None, Tone::Sac, Case::Upper)),
        'è' => Some((BaseVowel::E, Modifier::None, Tone::Huyen, Case::Lower)),
        'È' => Some((BaseVowel::E, Modifier::None, Tone::Huyen, Case::Upper)),
        'ẻ' => Some((BaseVowel::E, Modifier::None, Tone::Hoi, Case::Lower)),
        'Ẻ' => Some((BaseVowel::E, Modifier::None, Tone::Hoi, Case::Upper)),
        'ẽ' => Some((BaseVowel::E, Modifier::None, Tone::Nga, Case::Lower)),
        'Ẽ' => Some((BaseVowel::E, Modifier::None, Tone::Nga, Case::Upper)),
        'ẹ' => Some((BaseVowel::E, Modifier::None, Tone::Nang, Case::Lower)),
        'Ẹ' => Some((BaseVowel::E, Modifier::None, Tone::Nang, Case::Upper)),
        'ê' => Some((BaseVowel::E, Modifier::Circumflex, Tone::None, Case::Lower)),
        'Ê' => Some((BaseVowel::E, Modifier::Circumflex, Tone::None, Case::Upper)),
        'ế' => Some((BaseVowel::E, Modifier::Circumflex, Tone::Sac, Case::Lower)),
        'Ế' => Some((BaseVowel::E, Modifier::Circumflex, Tone::Sac, Case::Upper)),
        'ề' => Some((BaseVowel::E, Modifier::Circumflex, Tone::Huyen, Case::Lower)),
        'Ề' => Some((BaseVowel::E, Modifier::Circumflex, Tone::Huyen, Case::Upper)),
        'ể' => Some((BaseVowel::E, Modifier::Circumflex, Tone::Hoi, Case::Lower)),
        'Ể' => Some((BaseVowel::E, Modifier::Circumflex, Tone::Hoi, Case::Upper)),
        'ễ' => Some((BaseVowel::E, Modifier::Circumflex, Tone::Nga, Case::Lower)),
        'Ễ' => Some((BaseVowel::E, Modifier::Circumflex, Tone::Nga, Case::Upper)),
        'ệ' => Some((BaseVowel::E, Modifier::Circumflex, Tone::Nang, Case::Lower)),
        'Ệ' => Some((BaseVowel::E, Modifier::Circumflex, Tone::Nang, Case::Upper)),
        'i' => Some((BaseVowel::I, Modifier::None, Tone::None, Case::Lower)),
        'I' => Some((BaseVowel::I, Modifier::None, Tone::None, Case::Upper)),
        'í' => Some((BaseVowel::I, Modifier::None, Tone::Sac, Case::Lower)),
        'Í' => Some((BaseVowel::I, Modifier::None, Tone::Sac, Case::Upper)),
        'ì' => Some((BaseVowel::I, Modifier::None, Tone::Huyen, Case::Lower)),
        'Ì' => Some((BaseVowel::I, Modifier::None, Tone::Huyen, Case::Upper)),
        'ỉ' => Some((BaseVowel::I, Modifier::None, Tone::Hoi, Case::Lower)),
        'Ỉ' => Some((BaseVowel::I, Modifier::None, Tone::Hoi, Case::Upper)),
        'ĩ' => Some((BaseVowel::I, Modifier::None, Tone::Nga, Case::Lower)),
        'Ĩ' => Some((BaseVowel::I, Modifier::None, Tone::Nga, Case::Upper)),
        'ị' => Some((BaseVowel::I, Modifier::None, Tone::Nang, Case::Lower)),
        'Ị' => Some((BaseVowel::I, Modifier::None, Tone::Nang, Case::Upper)),
        'o' => Some((BaseVowel::O, Modifier::None, Tone::None, Case::Lower)),
        'O' => Some((BaseVowel::O, Modifier::None, Tone::None, Case::Upper)),
        'ó' => Some((BaseVowel::O, Modifier::None, Tone::Sac, Case::Lower)),
        'Ó' => Some((BaseVowel::O, Modifier::None, Tone::Sac, Case::Upper)),
        'ò' => Some((BaseVowel::O, Modifier::None, Tone::Huyen, Case::Lower)),
        'Ò' => Some((BaseVowel::O, Modifier::None, Tone::Huyen, Case::Upper)),
        'ỏ' => Some((BaseVowel::O, Modifier::None, Tone::Hoi, Case::Lower)),
        'Ỏ' => Some((BaseVowel::O, Modifier::None, Tone::Hoi, Case::Upper)),
        'õ' => Some((BaseVowel::O, Modifier::None, Tone::Nga, Case::Lower)),
        'Õ' => Some((BaseVowel::O, Modifier::None, Tone::Nga, Case::Upper)),
        'ọ' => Some((BaseVowel::O, Modifier::None, Tone::Nang, Case::Lower)),
        'Ọ' => Some((BaseVowel::O, Modifier::None, Tone::Nang, Case::Upper)),
        'ô' => Some((BaseVowel::O, Modifier::Circumflex, Tone::None, Case::Lower)),
        'Ô' => Some((BaseVowel::O, Modifier::Circumflex, Tone::None, Case::Upper)),
        'ố' => Some((BaseVowel::O, Modifier::Circumflex, Tone::Sac, Case::Lower)),
        'Ố' => Some((BaseVowel::O, Modifier::Circumflex, Tone::Sac, Case::Upper)),
        'ồ' => Some((BaseVowel::O, Modifier::Circumflex, Tone::Huyen, Case::Lower)),
        'Ồ' => Some((BaseVowel::O, Modifier::Circumflex, Tone::Huyen, Case::Upper)),
        'ổ' => Some((BaseVowel::O, Modifier::Circumflex, Tone::Hoi, Case::Lower)),
        'Ổ' => Some((BaseVowel::O, Modifier::Circumflex, Tone::Hoi, Case::Upper)),
        'ỗ' => Some((BaseVowel::O, Modifier::Circumflex, Tone::Nga, Case::Lower)),
        'Ỗ' => Some((BaseVowel::O, Modifier::Circumflex, Tone::Nga, Case::Upper)),
        'ộ' => Some((BaseVowel::O, Modifier::Circumflex, Tone::Nang, Case::Lower)),
        'Ộ' => Some((BaseVowel::O, Modifier::Circumflex, Tone::Nang, Case::Upper)),
        'ơ' => Some((BaseVowel::O, Modifier::Horn, Tone::None, Case::Lower)),
        'Ơ' => Some((BaseVowel::O, Modifier::Horn, Tone::None, Case::Upper)),
        'ớ' => Some((BaseVowel::O, Modifier::Horn, Tone::Sac, Case::Lower)),
        'Ớ' => Some((BaseVowel::O, Modifier::Horn, Tone::Sac, Case::Upper)),
        'ờ' => Some((BaseVowel::O, Modifier::Horn, Tone::Huyen, Case::Lower)),
        'Ờ' => Some((BaseVowel::O, Modifier::Horn, Tone::Huyen, Case::Upper)),
        'ở' => Some((BaseVowel::O, Modifier::Horn, Tone::Hoi, Case::Lower)),
        'Ở' => Some((BaseVowel::O, Modifier::Horn, Tone::Hoi, Case::Upper)),
        'ỡ' => Some((BaseVowel::O, Modifier::Horn, Tone::Nga, Case::Lower)),
        'Ỡ' => Some((BaseVowel::O, Modifier::Horn, Tone::Nga, Case::Upper)),
        'ợ' => Some((BaseVowel::O, Modifier::Horn, Tone::Nang, Case::Lower)),
        'Ợ' => Some((BaseVowel::O, Modifier::Horn, Tone::Nang, Case::Upper)),
        'u' => Some((BaseVowel::U, Modifier::None, Tone::None, Case::Lower)),
        'U' => Some((BaseVowel::U, Modifier::None, Tone::None, Case::Upper)),
        'ú' => Some((BaseVowel::U, Modifier::None, Tone::Sac, Case::Lower)),
        'Ú' => Some((BaseVowel::U, Modifier::None, Tone::Sac, Case::Upper)),
        'ù' => Some((BaseVowel::U, Modifier::None, Tone::Huyen, Case::Lower)),
        'Ù' => Some((BaseVowel::U, Modifier::None, Tone::Huyen, Case::Upper)),
        'ủ' => Some((BaseVowel::U, Modifier::None, Tone::Hoi, Case::Lower)),
        'Ủ' => Some((BaseVowel::U, Modifier::None, Tone::Hoi, Case::Upper)),
        'ũ' => Some((BaseVowel::U, Modifier::None, Tone::Nga, Case::Lower)),
        'Ũ' => Some((BaseVowel::U, Modifier::None, Tone::Nga, Case::Upper)),
        'ụ' => Some((BaseVowel::U, Modifier::None, Tone::Nang, Case::Lower)),
        'Ụ' => Some((BaseVowel::U, Modifier::None, Tone::Nang, Case::Upper)),
        'ư' => Some((BaseVowel::U, Modifier::Horn, Tone::None, Case::Lower)),
        'Ư' => Some((BaseVowel::U, Modifier::Horn, Tone::None, Case::Upper)),
        'ứ' => Some((BaseVowel::U, Modifier::Horn, Tone::Sac, Case::Lower)),
        'Ứ' => Some((BaseVowel::U, Modifier::Horn, Tone::Sac, Case::Upper)),
        'ừ' => Some((BaseVowel::U, Modifier::Horn, Tone::Huyen, Case::Lower)),
        'Ừ' => Some((BaseVowel::U, Modifier::Horn, Tone::Huyen, Case::Upper)),
        'ử' => Some((BaseVowel::U, Modifier::Horn, Tone::Hoi, Case::Lower)),
        'Ử' => Some((BaseVowel::U, Modifier::Horn, Tone::Hoi, Case::Upper)),
        'ữ' => Some((BaseVowel::U, Modifier::Horn, Tone::Nga, Case::Lower)),
        'Ữ' => Some((BaseVowel::U, Modifier::Horn, Tone::Nga, Case::Upper)),
        'ự' => Some((BaseVowel::U, Modifier::Horn, Tone::Nang, Case::Lower)),
        'Ự' => Some((BaseVowel::U, Modifier::Horn, Tone::Nang, Case::Upper)),
        'y' => Some((BaseVowel::Y, Modifier::None, Tone::None, Case::Lower)),
        'Y' => Some((BaseVowel::Y, Modifier::None, Tone::None, Case::Upper)),
        'ý' => Some((BaseVowel::Y, Modifier::None, Tone::Sac, Case::Lower)),
        'Ý' => Some((BaseVowel::Y, Modifier::None, Tone::Sac, Case::Upper)),
        'ỳ' => Some((BaseVowel::Y, Modifier::None, Tone::Huyen, Case::Lower)),
        'Ỳ' => Some((BaseVowel::Y, Modifier::None, Tone::Huyen, Case::Upper)),
        'ỷ' => Some((BaseVowel::Y, Modifier::None, Tone::Hoi, Case::Lower)),
        'Ỷ' => Some((BaseVowel::Y, Modifier::None, Tone::Hoi, Case::Upper)),
        'ỹ' => Some((BaseVowel::Y, Modifier::None, Tone::Nga, Case::Lower)),
        'Ỹ' => Some((BaseVowel::Y, Modifier::None, Tone::Nga, Case::Upper)),
        'ỵ' => Some((BaseVowel::Y, Modifier::None, Tone::Nang, Case::Lower)),
        'Ỵ' => Some((BaseVowel::Y, Modifier::None, Tone::Nang, Case::Upper)),
        _ => None,
    }
}

/// Checks if a character is a Vietnamese vowel (plain or accented).
#[inline]
pub fn is_vowel(ch: char) -> bool {
    decompose_vowel(ch).is_some()
}

/// Composes D with stroke (đ or Đ).
#[inline]
pub fn compose_dstroke(case: Case) -> char {
    match case {
        Case::Lower => 'đ',
        Case::Upper => 'Đ',
    }
}

/// Decomposes D-stroke character into Case, or None if not 'đ' / 'Đ'.
#[inline]
pub fn decompose_dstroke(ch: char) -> Option<Case> {
    match ch {
        'đ' => Some(Case::Lower),
        'Đ' => Some(Case::Upper),
        _ => None,
    }
}

/// Checks if a character is 'd' or 'D'.
#[inline]
pub fn is_plain_d(ch: char) -> bool {
    ch == 'd' || ch == 'D'
}

/// Checks if a character is 'đ' or 'Đ'.
#[inline]
pub fn is_dstroke(ch: char) -> bool {
    ch == 'đ' || ch == 'Đ'
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_compose_and_decompose_roundtrip() {
        let sample = "aáàảãạâấầẩẫậăắằẳẵặeéèẻẽẹêếềểễệiíìỉĩịoóòỏõọôốồổỗộơớờởỡợuúùủũụưứừửữựyýỳỷỹỵ";
        for ch in sample.chars() {
            let (base, modifier, tone, case) = decompose_vowel(ch).expect("should decompose");
            assert_eq!(case, Case::Lower);
            let composed = compose_vowel(base, modifier, tone, case).expect("should compose");
            assert_eq!(composed, ch);

            let ch_upper = ch.to_uppercase().next().unwrap();
            let (ubase, umod, utone, ucase) =
                decompose_vowel(ch_upper).expect("upper should decompose");
            assert_eq!(ucase, Case::Upper);
            assert_eq!(ubase, base);
            assert_eq!(umod, modifier);
            assert_eq!(utone, tone);
            let ucomposed = compose_vowel(ubase, umod, utone, ucase).expect("upper should compose");
            assert_eq!(ucomposed, ch_upper);
        }
    }

    #[test]
    fn test_nfc_invariant() {
        for ch in
            "aáàảãạâấầẩẫậăắằẳẵặeéèẻẽẹêếềểễệiíìỉĩịoóòỏõọôốồổỗộơớờởỡợuúùủũụưứừửữựyýỳỷỹỵđĐ".chars()
        {
            assert!(!('\u{0300}'..='\u{036F}').contains(&ch));
            assert_eq!(ch.len_utf8(), ch.to_string().len());
        }
    }

    #[test]
    fn test_dstroke() {
        assert_eq!(compose_dstroke(Case::Lower), 'đ');
        assert_eq!(compose_dstroke(Case::Upper), 'Đ');
        assert_eq!(decompose_dstroke('đ'), Some(Case::Lower));
        assert_eq!(decompose_dstroke('Đ'), Some(Case::Upper));
        assert_eq!(decompose_dstroke('d'), None);
    }
}
