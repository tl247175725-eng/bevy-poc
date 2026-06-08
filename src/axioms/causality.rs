use std::collections::VecDeque;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CausalEvent {
    pub tick: u64,
    pub cause_entity_id: u64,
    pub cause_tag: String,
    pub effect_entity_id: u64,
    pub effect_description: String,
}

pub enum CausalStorage {
    Full { events: Vec<CausalEvent> },
    Ring { buf: VecDeque<CausalEvent>, cap: usize },
    Off,
}

impl CausalStorage {
    pub fn push(&mut self, event: CausalEvent) {
        match self {
            CausalStorage::Full { events } => events.push(event),
            CausalStorage::Ring { buf, cap } => {
                if buf.len() >= *cap {
                    buf.pop_front();
                }
                buf.push_back(event);
            }
            CausalStorage::Off => {}
        }
    }

    pub fn for_mode(is_smoke_test: bool, is_debug: bool) -> Self {
        if is_smoke_test {
            CausalStorage::Ring {
                buf: VecDeque::with_capacity(2000),
                cap: 2000,
            }
        } else if is_debug {
            CausalStorage::Full {
                events: Vec::new(),
            }
        } else {
            CausalStorage::Off
        }
    }
}

impl Default for CausalStorage {
    fn default() -> Self {
        CausalStorage::Off
    }
}
