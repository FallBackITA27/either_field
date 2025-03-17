pub(crate) fn get_alpha(n: usize) -> String {
    const ALPHABET: [char; 26] = [
        'A', 'B', 'C', 'D', 'E', 'F', 'G', 'H', 'I', 'J', 'K', 'L', 'M', 'N', 'O', 'P', 'Q', 'R',
        'S', 'T', 'U', 'V', 'W', 'X', 'Y', 'Z',
    ];
    if n < 26 {
        return ALPHABET[n].to_string();
    }
    let mut x = get_alpha((n / 26) - 1);
    x.push(ALPHABET[n % 26]);
    x
}