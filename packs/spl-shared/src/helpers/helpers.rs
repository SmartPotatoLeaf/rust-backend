pub fn lpad(input: &str, total_width: usize, pad_char: char) -> String {
    if input.len() >= total_width {
        return input.to_string();
    }

    let padding = total_width - input.len();
    pad_char.to_string().repeat(padding) + input
}