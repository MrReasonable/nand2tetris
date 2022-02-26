trait Parse {
    fn has_more_commands() -> bool;
    fn advance();
    fn logical_line_no() -> isize;
    fn line_no() -> isize;
}
