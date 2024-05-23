use std::str;

#[derive(Clone, Debug)]
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


pub struct Column {
    pub name: String,
    pub column_type: ColumnType
}

impl Column {
    pub fn new(name: String, column_type: ColumnType) -> Column {
        Column {name, column_type}
    }
}

pub struct TableSchema {
    pub name: String,
    pub first_page_id: u32,
    pub columns: Vec<Column>
}

impl TableSchema {
    pub fn new(name: String, columns: Vec<Column>, first_page_id: u32) -> TableSchema {
        TableSchema {name, first_page_id, columns}
    }
    pub fn serialize(&self) -> Vec<u8> {
        // layout in bytes
        // [length of name] [name] [page id of first page] [number of columns] [column entries]+
        // where each column entry is [column type][column name length][column name]
        let mut res = Vec::new();

        let name_length = self.name.len();
        let name_length = u8::try_from(name_length).unwrap(); // name length has to fit in u8

        res.push(name_length);
        res.extend(self.name.as_bytes());

        res.extend(self.first_page_id.to_le_bytes());

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

    pub fn deserialize(data: Vec<u8>) -> TableSchema {
        // see serialize
        let mut bytes_read = 0;
        
        let name_len = usize::from(data[0]);
        bytes_read += 1;

        let name = &data[bytes_read..bytes_read+name_len];
        let name: String = str::from_utf8(name).unwrap().to_string();
        bytes_read += name_len;

        let first_page_id = &data[bytes_read..bytes_read+4];
        let first_page_id = u32::from_le_bytes(first_page_id.try_into().unwrap());
        bytes_read += 4;

        let num_col = usize::from(data[bytes_read]);
        bytes_read += 1;

        let mut columns = Vec::new();

        loop {
            if columns.len() == num_col {
                break;
            }
            let col_type = decode_column_type(data[bytes_read]);
            bytes_read += 1;
            let col_name_len = usize::from(data[bytes_read]);
            bytes_read += 1;
            let col_name = str::from_utf8(&data[bytes_read..bytes_read+col_name_len]).unwrap().to_string();
            bytes_read += col_name_len;

            columns.push(Column::new(col_name, col_type));
        }


        TableSchema::new(name, columns, first_page_id)
    }
}

#[cfg(test)]
mod tests {
    use super::{Column, ColumnType, TableSchema};

    
    #[test]
    fn simple_serialize() {
        let c = Column::new("1".to_owned(), ColumnType::Bool);
        let t = TableSchema::new("0".to_owned(), vec![c], 1);
        let s = t.serialize();
        let expected: Vec<u8> = vec![
            1, //len of table name 
            48, //name
            1,0,0,0, // page id 1 (u32) in LE
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
            1,0,0,0, // page id 1 (u32) in LE
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
