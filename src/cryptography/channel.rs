use std::collections::VecDeque;

pub struct Channel {
    packets: VecDeque<Vec<u8>>,
}

impl Channel {
    pub fn new() -> Self {
        Self {
            packets: VecDeque::new(),
        }
    }

    pub fn send(&mut self, ciphertext: Vec<u8>) {
        self.packets.push_back(ciphertext);
    }

    pub fn receive(&mut self) -> Option<Vec<u8>> {
        self.packets.pop_front()
    }
}