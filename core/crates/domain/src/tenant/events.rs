use crate::{EventType, EventVersion};

pub enum EventV1 {
    Created { name: String },
}

impl EventType for EventV1 {
    fn get_event_type(&self) -> &str {
        match self {
            EventV1::Created { .. } => "TenantCreated",
        }
    }
}

impl EventVersion for EventV1 {
    const VERSION: u8 = 1;
}

impl PartialEq for EventV1 {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (EventV1::Created { name: name1 }, EventV1::Created { name: name2 }) => name1 == name2,
        }
    }
}

impl PartialEq<&str> for EventV1 {
    fn eq(&self, other: &&str) -> bool {
        self.get_event_type() == *other
    }
}

impl Eq for EventV1 {}
