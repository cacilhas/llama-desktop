pub fn format_input_to_output(inp: impl Into<String>) -> String {
    let inp: String = inp.into();
    let mut out = "> ".to_string();
    for c in inp.chars() {
        out.push(c);
        if c == '\n' {
            out.push_str("> ");
        }
    }
    out.push('\n');
    out
}

pub const HR: &str = "\n\n-----\n";
