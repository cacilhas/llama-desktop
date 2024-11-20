pub fn format_input_to_output(inp: impl ToString) -> String {
    let mut out = "> ".to_string();
    for c in inp.to_string().chars() {
        out.push(c);
        if c == '\n' {
            out.push_str("> ");
        }
    }
    out.push('\n');
    out
}

pub const HR: &str = "\n\n-----\n";
