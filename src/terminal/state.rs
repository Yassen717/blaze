#[derive(Clone, Debug, PartialEq)]
pub struct TerminalLine {
    pub content: String,
    pub line_type: LineType,
}

#[derive(Clone, Debug, PartialEq)]
pub enum LineType {
    Command,
    Output,
    Error,
    System,
}
