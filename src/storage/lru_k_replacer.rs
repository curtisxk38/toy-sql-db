
use std::collections::HashMap;
use std::time::SystemTime;
use std::collections::VecDeque;
use std::collections::HashSet;

use super::buffer_pool::FrameId;
use super::buffer_pool::PageId;


pub struct LRUKReplacer {
    num_frames: usize,
    k: usize,
    //recently_evicted: HashMap<usize, SystemTime>,
    access_histories: HashMap<FrameId, VecDeque<SystemTime>>,
    evictable: HashSet<FrameId>,
}

pub struct EvictionError {}

impl LRUKReplacer {
    pub fn new(num_frames: usize, k: usize) -> LRUKReplacer {
        LRUKReplacer {num_frames, k, access_histories: HashMap::new(), evictable: HashSet::new()}
    }

    pub fn evict(&self) -> Result<FrameId, EvictionError> {
        // returns frame id to evict or error on failure to evict
        let mut to_evict: Option<FrameId> = None;
        let mut to_evict_access = SystemTime::now();
        for evictable_frame in &self.evictable {
             match self.access_histories.get(&evictable_frame) {
                Some(access_history) => {
                    // None signifies inf (aka oldest time possible)
                    let this_frame_access = if access_history.len() == self.k {access_history.front()} else { None };
                    
                    match this_frame_access {
                        Some(this_time) => {
                            if this_time < &to_evict_access {
                                // evictable_frame is older than to_evict
                                to_evict = Some(*evictable_frame);
                                to_evict_access = *this_time;
                            }
                        },
                        None => {
                            // TODO in case of ties between multiple frames with inf, use LRU
                            //  instead of choosing first one we encounter like we do here:

                            // evictable_frame is older to_evict
                            //  since evictable_frame has oldest time possible,
                            // so we can just evict it, nothing else is older
                            return Ok(*evictable_frame);
                        }
                    }
                },
                None => {}
             };
        };
        match to_evict {
            Some(page_id) => Ok(page_id),
            None => Err(EvictionError {})
        }
    }

    pub fn record_access(&mut self, frame_id: FrameId) {
        // record that given frame was accessed
        // call after page is pinned in buffer pool
        let now = SystemTime::now();
        match self.access_histories.get_mut(&frame_id) {
            Some(access_history) => {
                if access_history.len() >= self.k {
                    // remove oldest
                    access_history.pop_front();
                }
                access_history.push_back(now);
            }
            None => {
                let new_history = VecDeque::from([now]);
                self.access_histories.insert(frame_id, new_history);
            }
        }

    }

    pub fn remove(&mut self, frame_id: FrameId) {
        // clear access history for this frame
        // call after buffer pool deletes the frame
        self.evictable.remove(&frame_id);
        self.access_histories.remove(&frame_id);
    }

    pub fn set_evictable(&mut self, frame_id: FrameId, set_evictable: bool) {
        // when pin count of a page reaches 0, its corresponding frame is marked evictable and replacer's size is incremented.
        if set_evictable {
            self.evictable.insert(frame_id);
        } else {
            self.evictable.remove(&frame_id);
        }
    }

    pub fn size(&self) -> usize {
        // return numer of evictable frames
        return self.evictable.len()
    }
}