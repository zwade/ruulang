pub struct CodegenHelper<'a> {
    indent_with: &'a str,
    line_terminator: &'a str,

    indent: usize,
    buffer: String,
    pending_token: bool,
    pending_newline: bool,
}

impl<'a> CodegenHelper<'a> {
    pub fn new(indent_with: &'a str, line_terminator: &'a str) -> Self {
        Self {
            indent_with,
            line_terminator,
            indent: 0,
            buffer: String::new(),
            pending_token: false,
            pending_newline: false,
        }
    }

    pub fn with_indent(&mut self, op: impl FnOnce(&mut Self)) {
        self.indent += 1;
        op(self);
        self.indent -= 1;
    }

    pub fn with_duouble_quote(&mut self, op: impl FnOnce(&mut Self)) {
        self.write("\"");
        op(self);
        self.write("\"");
    }

    pub fn with_parens(&mut self, op: impl FnOnce(&mut Self)) {
        self.write_symbol("(");
        op(self);
        self.write_symbol(")");
    }

    pub fn iter_and_join<T, Item, F>(&mut self, list: T, sep: &str, mut op: F)
    where
        T: IntoIterator<Item = Item>,
        F: FnMut(&mut Self, &Item),
    {
        let as_vec = list.into_iter().collect::<Vec<_>>();
        if as_vec.len() == 0 {
            return;
        }

        for item in as_vec.iter().take(as_vec.len() - 1) {
            op(self, item);
            self.write_symbol(sep);
        }

        op(self, as_vec.last().unwrap());
    }

    pub fn write_token(&mut self, token: &str) {
        self.write_helper(token, true);
        self.pending_token = true;
    }

    pub fn write_symbol(&mut self, symbol: &str) {
        self.write_helper(symbol, false);
    }

    pub fn write(&mut self, data: &str) {
        self.write_helper(data, true);
    }

    pub fn write_line(&mut self, data: Option<&str>) {
        if let Some(data) = data {
            self.write_helper(data, true);
        }

        self.write_symbol(self.line_terminator);
        self.pending_newline = true;
    }

    pub fn serialize(self) -> String {
        return self.buffer;
    }

    fn write_helper(&mut self, data: &str, respect_token: bool) {
        if self.pending_newline {
            for _ in 0..self.indent {
                self.buffer.push_str(self.indent_with);
            }
        }

        if self.pending_token && respect_token {
            self.buffer.push_str(" ");
        }

        self.buffer.push_str(data);
        self.pending_token = false;
        self.pending_newline = false;
    }
}
