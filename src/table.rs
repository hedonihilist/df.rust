use std::collections::HashMap;

macro_rules! lines {
    ($( $x:expr ),* $(,)?) => {
        {
            let mut v = vec![];
            $(
                v.push(($x).to_string());
            )*
            v.join("\n")
        }
    }
}

#[derive(Default)]
pub struct Table {
    fields: Vec<String>,
    content: Vec<Vec<String>>,
    column_width: Vec<usize>,
    aligns: HashMap<String, FieldAlign>,
    align: FieldAlign,
}

#[derive(Clone, Copy)]
pub enum FieldAlign {
    Left,
    Right,
    Center,
}

impl Default for FieldAlign {
    fn default() -> FieldAlign {
        FieldAlign::Left
    }
}

impl Table {
    pub fn change_field_name(&mut self, orig: &str, new: &str) {
        for i in 0..self.fields.len() {
            if self.fields[i].eq(orig) {
                self.fields[i] = new.to_owned();
            }
        }
        self.update_column_width();
    }

    fn update_column_width(&mut self) {
        for i in 0..self.column_width.len() {
            self.column_width[i] = 0;
        }

        for (i,field) in self.fields.iter().enumerate() {
            self.column_width[i] = std::cmp::max(self.column_width[i], field.len());
        }

        for content in self.content.iter() {
            for (i, v) in content.iter().enumerate() {
                self.column_width[i] = std::cmp::max(self.column_width[i], v.len());
            }
        }
    }

    pub fn new<T: AsRef<str>>(fields: &[T]) -> Table {
        Table {
            fields: fields.iter().map(|x| x.as_ref().to_string()).collect(),
            column_width: fields.iter().map(|x| x.as_ref().len()).collect(),
            ..Default::default()
        }
    }

    pub fn len(&self) -> usize {
        self.content.len()
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    pub fn add_row<T: AsRef<str>>(&mut self, row: &[T]) {
        if row.len() != self.fields.len() {
            panic!("size doesn't match");
        }
        let mut r = vec![];
        for (i, x) in row.iter().enumerate() {
            self.column_width[i] = std::cmp::max(self.column_width[i], x.as_ref().len());
            r.push(x.as_ref().to_string());
        }
        self.content.push(r);
    }

    /**
     * set align for specific field
     */
    pub fn set_field_align(&mut self, field: &str, align: FieldAlign) {
        self.aligns.insert(field.to_string(), align);
    }

    /**
     * set align for all fields
     */
    pub fn set_align(&mut self, align: FieldAlign) {
        self.align = align;
    }

    pub fn padding(s: &str, width: usize, align: &FieldAlign) -> String {
        if width <= s.len() {
            return s.to_string();
        }
        let delta = width - s.len();
        match &align {
            FieldAlign::Left => format!("{}{}", s, Self::whitespace(delta)),
            FieldAlign::Right => format!("{}{}", Self::whitespace(delta), s),
            FieldAlign::Center => format!(
                "{}{}{}",
                Self::whitespace(delta / 2),
                s,
                Self::whitespace(delta - delta / 2)
            ),
        }
    }

    pub fn whitespace(n: usize) -> String {
        (0..n).map(|_| ' ').collect::<String>()
    }

    /**
     * ??????????????????field
     */
    pub fn to_string_partial<T: AsRef<str>>(&self, field_list: &[T]) -> String {
        // get field -> idx mapping
        let mut field_idx: HashMap<&str, usize> = HashMap::new();
        for (i, x) in self.fields.iter().enumerate() {
            field_idx.insert(x, i);
        }

        // get column align
        let mut align_list: Vec<&FieldAlign> = vec![];
        for field in self.fields.iter() {
            align_list.push(self.aligns.get(field).unwrap_or(&self.align));
        }

        // get idx list of from field_list
        let mut idx_list: Vec<usize> = vec![];
        for field in field_list.iter() {
            if let Some(&idx) = field_idx.get(field.as_ref()) {
                idx_list.push(idx);
            } else {
                panic!("field {} doesn't exist in table", field.as_ref());
            }
        }

        let mut rows: Vec<String> = vec![];

        // first row
        let mut header: Vec<String> = vec![];
        for &idx in idx_list.iter() {
            header.push(Self::padding(
                &self.fields[idx],
                self.column_width[idx],
                align_list[idx],
            ));
        }
        rows.push(header.join(&" ".to_string()));

        // content
        for r in self.content.iter() {
            let mut row = vec![];
            for &idx in idx_list.iter() {
                row.push(Self::padding(
                    &r[idx],
                    self.column_width[idx],
                    align_list[idx],
                ));
            }
            rows.push(row.join(&" ".to_string()));
        }

        rows.join(&"\n".to_string())
    }
}

impl ToString for Table {
    /*
    fn to_string(&self) -> String {
        // get column align
        let mut align_list: Vec<&FieldAlign> = vec![];
        for field in self.fields.iter() {
            align_list.push(self.aligns.get(field).unwrap_or(&self.align));
        }

        let mut rows: Vec<String> = vec![];

        // first row
        let mut header: Vec<String> = vec![];
        for (i, field) in self.fields.iter().enumerate() {
            header.push(Self::padding(field, self.column_width[i], align_list[i]));
        }
        rows.push(header.join(&" ".to_string()));

        // content
        for r in self.content.iter() {
            let mut row = vec![];
            for (i, field) in r.iter().enumerate() {
                row.push(Self::padding(field, self.column_width[i], align_list[i]));
            }
            rows.push(row.join(&" ".to_string()));
        }

        rows.join(&"\n".to_string())
    }
     */
    fn to_string(&self) -> String {
        self.to_string_partial(&self.fields)
    }
}

#[cfg(test)]
mod tests {
    use crate::table;
    use crate::table::FieldAlign;
    use crate::table::Table;

    #[test]
    fn test() {
        assert!(true);
    }

    #[test]
    fn test_padding() {
        assert_eq!(
            table::Table::padding("left", 8, &table::FieldAlign::Left),
            "left    "
        );
        assert_eq!(
            table::Table::padding("right", 8, &table::FieldAlign::Right),
            "   right"
        );
        assert_eq!(
            table::Table::padding("center", 8, &table::FieldAlign::Center),
            " center "
        );
        assert_eq!(
            table::Table::padding("center", 9, &table::FieldAlign::Center),
            " center  "
        );
    }

    #[test]
    fn test_whitespace() {
        assert_eq!(3, table::Table::whitespace(3).len());
        assert_eq!(0, table::Table::whitespace(0).len());
        assert_eq!(10, table::Table::whitespace(10).len());
        assert_eq!(13, table::Table::whitespace(13).len());
    }

    #[test]
    fn test_table_to_string() {
        let header = vec!["Username", "age", "email"];
        let mut table = Table::new(&header);
        table.set_align(FieldAlign::Left);
        table.set_field_align("age", FieldAlign::Right);
        table.add_row(&vec!["Harry", "15", "harry@163.com"]);
        table.add_row(&vec!["Ron", "15", "ron@163.com"]);
        table.add_row(&vec!["Hermione", "15", "hermione@163.com"]);
        assert_eq!(
            table.to_string(),
            lines!(
                "Username age email           ",
                "Harry     15 harry@163.com   ",
                "Ron       15 ron@163.com     ",
                "Hermione  15 hermione@163.com",
            )
        );
    }

    #[test]
    fn test_table_to_string_partial() {
        let header = vec!["Username", "age", "email"];
        let mut table = Table::new(&header);
        table.set_align(FieldAlign::Left);
        table.set_field_align("age", FieldAlign::Right);
        table.add_row(&vec!["Harry", "15", "harry@163.com"]);
        table.add_row(&vec!["Ron", "15", "ron@163.com"]);
        table.add_row(&vec!["Hermione", "15", "hermione@163.com"]);
        assert_eq!(
            table.to_string_partial(&vec!["Username", "email"]),
            lines!(
                "Username email           ",
                "Harry    harry@163.com   ",
                "Ron      ron@163.com     ",
                "Hermione hermione@163.com",
            )
        );
    }

    #[test]
    fn test_lines() {
        assert_eq!("a\nb\nc\nd", lines!("a", "b", "c", "d"));
    }
}
