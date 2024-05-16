use std::{cell::RefCell, rc::Rc};

use crate::config::config::PAGE_SIZE;


/**
 * Slotted page format:
 *  ---------------------------------------------------------
 *  | HEADER | ... FREE SPACE ... | ... INSERTED TUPLES ... |
 *  ---------------------------------------------------------
 *                                ^
 *                                free space pointer
 *
 *  Header format (size in bytes):
 *  ----------------------------------------------------------------------------
 *  | NextPageId (4)| NumTuples(2) | NumDeletedTuples(2) |
 *  ----------------------------------------------------------------------------
 *  ----------------------------------------------------------------
 *  | Tuple_1 entry (4) | Tuple_2 entry (4) | ... |
 *  ----------------------------------------------------------------
 * 
 * 
 *  Tuple entry format:
 *  | tuple offset (4) | tuple size (4)  |
 */

const TABLE_PAGE_HEADER_SIZE: usize = 8; // in bytes

const SLOT_ARRAY_ENTRY_SIZE: usize = 4;

#[derive(Eq, Hash, PartialEq, Clone)]
pub struct TupleId(usize);

struct TablePage<'a> {
    next_page_id: u32,
    num_tuples: u16,
    num_deleted_tuples: u16,
    page_data: Rc<RefCell<&'a mut [u8]>>,
}

impl <'a> TablePage<'a> {
    pub fn new(page_data: Rc<RefCell<&'a mut [u8]>>) -> TablePage<'a> {
        let d = page_data.borrow();

        let next_page_id: &[u8] = &d[0..4];
        let next_page_id = u32::from_le_bytes(next_page_id.try_into().unwrap());

        let num_tuples: &[u8] = &d[4..6];
        let num_tuples = u16::from_le_bytes(num_tuples.try_into().unwrap());

        let num_deleted_tuples: &[u8] = &d[6..8];
        let num_deleted_tuples = u16::from_le_bytes(num_deleted_tuples.try_into().unwrap());

        std::mem::drop(d); // this is kinda dumb but works

        TablePage { next_page_id, num_tuples, num_deleted_tuples, page_data }
    }

    fn get_num_tuples(&self) -> u16 {
        self.num_tuples
    }

    fn set_num_tuples(&mut self, num_tuples: u16) {
        self.num_tuples = num_tuples;
        let num_tuples = num_tuples.to_le_bytes();
        self.page_data.borrow_mut()[4..6].copy_from_slice(&num_tuples);
    }

    pub fn get_next_tuple_offset(&self, tuple: &Vec<u8>) -> Option<usize> {
        let num_tuples: usize = self.get_num_tuples().into();
        let slot_end_offset: usize;
        if num_tuples > 0 {
        
            let last_slot_array_entry_offset = TABLE_PAGE_HEADER_SIZE + (num_tuples - 1) * SLOT_ARRAY_ENTRY_SIZE;
            let last_slot_array_entry = &self.page_data.borrow()[last_slot_array_entry_offset..last_slot_array_entry_offset+2];

            let last_slot_array_entry = u16::from_le_bytes(last_slot_array_entry.try_into().unwrap());
            slot_end_offset = last_slot_array_entry.try_into().unwrap();
            
        } else {
            slot_end_offset = PAGE_SIZE;
        }
        let proposed_tuple_offset = slot_end_offset - tuple.len();
        if TABLE_PAGE_HEADER_SIZE + (num_tuples + 1) * SLOT_ARRAY_ENTRY_SIZE < proposed_tuple_offset {
            Some(proposed_tuple_offset)
        } else {
            None
        }
    }

    pub fn insert_tuple(&mut self, tuple: Vec<u8>) -> Option<TupleId> {
        match self.get_next_tuple_offset(&tuple) {
            Some(tuple_offset) => {
                let tuple_id = self.get_num_tuples();
                self.set_num_tuples(tuple_id+1);
                
                // copy data in
                self.page_data.borrow_mut()[tuple_offset..tuple_offset+tuple.len()].copy_from_slice(&tuple);

                // update slot array
                let tuple_offset_bytes: u32 = tuple_offset.try_into().unwrap();
                let tuple_offset_bytes = tuple_offset_bytes.to_le_bytes();
                
                let new_slot_array_index: usize = TABLE_PAGE_HEADER_SIZE + Into::<usize>::into(tuple_id) * SLOT_ARRAY_ENTRY_SIZE;
                self.page_data.borrow_mut()[new_slot_array_index..new_slot_array_index+SLOT_ARRAY_ENTRY_SIZE].copy_from_slice(&tuple_offset_bytes);

                Some(TupleId(tuple_id.into()))
            },
            None => None,
        }
    }

    pub fn get_tuple(&self, tuple_id: TupleId) -> Vec<u8> {
        if tuple_id.0 < self.get_num_tuples().into() {
            let slot_index = TABLE_PAGE_HEADER_SIZE + tuple_id.0 * SLOT_ARRAY_ENTRY_SIZE;
            let tuple_offset = &self.page_data.borrow()[slot_index..slot_index+SLOT_ARRAY_ENTRY_SIZE];
            let tuple_offset: usize = u32::from_le_bytes(tuple_offset.try_into().unwrap()).try_into().unwrap();
            let tuple_size = 0; // TODO fix

            let tuple = self.page_data.borrow()[tuple_offset..tuple_offset+tuple_size].to_vec();

            tuple
        } else {
            panic!("invalid tuple id for this page");
         }
    }

}

#[cfg(test)]
mod tests {
    use std::{cell::RefCell, rc::Rc};

    use crate::config::config::PAGE_SIZE;

    use super::{TablePage};

    
    #[test]
    fn test_get_offset() {
        let mut page_data: Vec<u8> = vec![
            0,0,0,0, //next page id
            1,0, // num tuples
            0,0, //num deleted tuples
            // start of slot array
            0xC0, 0x0F, // offset = 4032
            0x40, 0x00 // size = 64
        ];
        let p = TablePage::new(Rc::from(RefCell::from(&mut page_data[..])));
        assert_eq!(p.get_num_tuples(), 1);
        
        let tuple: Vec<u8> = vec![0; 32];

        let offset = p.get_next_tuple_offset(&tuple).unwrap();

        let expected = 4000;

        assert_eq!(offset, expected);
    }
}