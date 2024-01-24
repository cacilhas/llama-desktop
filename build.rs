use slint_build::CompileError;

fn main() -> Result<(), CompileError> {
    slint_build::compile("ui/appwindow.slint")
}
