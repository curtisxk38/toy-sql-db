use std::str;

pub enum ColumnType {
    Int,
    Bool
}

pub fn encode_column_type(t: &ColumnType) -> u8 {
    match t {
        ColumnType::Int => 0,
        ColumnType::Bool => 1,
    }
}

pub fn decode_column_type(u: u8) -> ColumnType {
    match u {
        0 => ColumnType::Int,
        1 => ColumnType::Bool,
        _ => panic!("unexpected column type")
    }
}


struct Column {
    pub name: String,
    pub column_type: ColumnType
}

impl Column {
    pub fn new(name: String, column_type: ColumnType) -> Column {
        Column {name, column_type}
    }
}

struct TableSchema {
    pub name: String,
    pub columns: Vec<Column>
}

impl TableSchema {
    fn new(name: String, columns: Vec<Column>) -> TableSchema {
        TableSchema {name, columns}
    }
    fn serialize(&self) -> Vec<u8> {
        // layout in bytes
        // [length of name] [name] [number of columns] [column entries]+
        // where each column entry is [column type][column name length][column name]
        let mut res = Vec::new();

        let name_length = self.name.len();
        let name_length = u8::try_from(name_length).unwrap(); // name length has to fit in u8

        res.push(name_length);
        res.extend(self.name.as_bytes());

        let num_columns = u8::try_from(self.columns.len()).unwrap();
        res.push(num_columns);

        for column in &self.columns {
            res.push(encode_column_type(&column.column_type));
            let col_len = u8::try_from(column.name.len()).unwrap();
            res.push(col_len);
            res.extend(column.name.as_bytes());
        }

        res
    }

    fn deserialize(data: Vec<u8>) -> TableSchema {
        // see serialize
        let name_len = usize::from(data[0]);
        let name = &data[1..name_len+1];
        let name: String = str::from_utf8(name).unwrap().to_string();
        let num_col = usize::from(data[name_len+1]);
        let mut columns = Vec::new();

        let mut reading_from = name_len+2;
        loop {
            if columns.len() == num_col {
                break;
            }
            let col_type = decode_column_type(data[reading_from]);
            reading_from += 1;
            let col_name_len = usize::from(data[reading_from]);
            reading_from += 1;
            let col_name = str::from_utf8(&data[reading_from..reading_from+col_name_len]).unwrap().to_string();
            reading_from += col_name_len;

            columns.push(Column::new(col_name, col_type));
        }


        TableSchema::new(name, columns)
    }
}

#[cfg(test)]
mod tests {
    use super::{Column, ColumnType, TableSchema};

    
    #[test]
    fn simple_serialize() {
        let c = Column::new("1".to_owned(), ColumnType::Bool);
        let t = TableSchema::new("0".to_owned(), vec![c]);
        let s = t.serialize();
        let expected: Vec<u8> = vec![
            1, //len of table name 
            48, //name
            1, //number of col
            1, //col type
            1, // len of col name
            49, // col name
            ];
        assert_eq!(s, expected)
    }

    #[test]
    fn simple_deserialize() {
        let data: Vec<u8> = vec![
            1, //len of table name 
            48, //name
            1, //number of col
            1, //col type
            1, // len of col name
            49, // col name
            ];
        let t = TableSchema::deserialize(data);

        assert_eq!(t.name, String::from("0"));
        assert_eq!(t.columns.len(), 1);
        assert_eq!(t.columns[0].name, "1");
        assert!(matches!(t.columns[0].column_type, ColumnType::Bool));
    }

}
