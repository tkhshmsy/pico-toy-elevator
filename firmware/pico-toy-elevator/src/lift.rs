#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum FloorState {
    Floor1st = 0,
    Floor2nd,
    Floor3rd,
    Floor4th,
    Floor5th,
    Floor6th,
    Floor7th,
    Floor8th,
    FloorMax,
}

impl FloorState {
    pub fn up(&self) -> FloorState {
        match self {
            FloorState::Floor1st => FloorState::Floor2nd,
            FloorState::Floor2nd => FloorState::Floor3rd,
            FloorState::Floor3rd => FloorState::Floor4th,
            FloorState::Floor4th => FloorState::Floor5th,
            FloorState::Floor5th => FloorState::Floor6th,
            FloorState::Floor6th => FloorState::Floor7th,
            FloorState::Floor7th => FloorState::Floor8th,
            _ => self.clone(),
        }
    }

    pub fn down(&self) -> FloorState {
        match self {
            FloorState::Floor2nd => FloorState::Floor1st,
            FloorState::Floor3rd => FloorState::Floor2nd,
            FloorState::Floor4th => FloorState::Floor3rd,
            FloorState::Floor5th => FloorState::Floor4th,
            FloorState::Floor6th => FloorState::Floor5th,
            FloorState::Floor7th => FloorState::Floor6th,
            FloorState::Floor8th => FloorState::Floor7th,
            _ => self.clone(),
        }
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum LiftState {
    Chime,
    Arrived,
    Opening,
    Waiting,
    Closing,
    Checking,
    Checked,
    Moving,
}

impl LiftState {
    // pub fn new() -> LiftState {
    //     LiftState::Opening,
    // }

    pub fn next(&self) -> LiftState {
        match self {
            LiftState::Chime => LiftState::Arrived,
            LiftState::Arrived => LiftState::Opening,
            LiftState::Opening => LiftState::Waiting,
            LiftState::Waiting => LiftState::Closing,
            LiftState::Closing => LiftState::Checking,
            LiftState::Checking => LiftState::Moving,
            LiftState::Moving => LiftState::Chime,
            _ => self.clone(),
        }
    }

    pub fn open(&self) -> LiftState {
        match self {
            LiftState::Closing => LiftState::Opening,
            _ => self.clone(),
        }
    }

    pub fn close(&self) -> LiftState {
        match self {
            LiftState::Opening => LiftState::Closing,
            LiftState::Waiting => LiftState::Closing,
            _ => self.clone(),
        }
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum Directions {
    Up,
    Down,
    None,
}

pub struct Lift {
    floor_state: FloorState,
    lift_state: LiftState,
    direction: Directions,
    floor_keys: [bool; FloorState::FloorMax as usize],
}

impl Lift {
    pub fn new() -> Lift {
        Lift {
            floor_state: FloorState::Floor1st,
            lift_state: LiftState::Arrived,
            direction: Directions::None,
            floor_keys: [false; FloorState::FloorMax as usize],
        }
    }

    pub fn floor_state(&self) -> &FloorState {
        &self.floor_state
    }

    pub fn lift_state(&self) -> &LiftState {
        &self.lift_state
    }

    pub fn direction(&self) -> &Directions {
        &self.direction
    }

    pub fn set_floor_key(&mut self, floor: FloorState, key: bool) -> bool {
        if key {
            if self.floor_state() == &floor {
                return false;
            }
        }
        let prev = self.floor_keys[floor as usize];
        self.floor_keys[floor as usize] = key;
        key != prev
    }

    fn has_current_by_key(&self) -> bool {
        let cur = self.floor_state as i32;
        self.floor_keys[cur as usize]
    }

    fn has_upper_by_key(&self) -> bool {
        let cur = self.floor_state as i32;
        for (f, k) in self.floor_keys.iter().enumerate() {
            if *k && cur < (f as i32) {
                return true;
            }
        }
        false
    }

    fn has_lower_by_key(&self) -> bool {
        let cur = self.floor_state as i32;
        for (f, k) in self.floor_keys.iter().enumerate() {
            if *k && cur > (f as i32) {
                return true;
            }
        }
        false
    }

    fn update_direction(&mut self) {
        match self.direction {
            Directions::Up => {
                if !self.has_upper_by_key() {
                    if self.has_lower_by_key() {
                        self.direction = Directions::Down;
                    } else {
                        self.direction = Directions::None;
                    }
                }
            }
            Directions::Down => {
                if !self.has_lower_by_key() {
                    if self.has_upper_by_key() {
                        self.direction = Directions::Up;
                    } else {
                        self.direction = Directions::None;
                    }
                }
            }
            Directions::None => {
                if self.has_upper_by_key() {
                    self.direction = Directions::Up;
                } else if self.has_lower_by_key() {
                    self.direction = Directions::Down;
                }
            }
        }
    }

    pub fn next(&mut self) {
        let is_next = match self.lift_state {
            LiftState::Chime => true,
            LiftState::Arrived => self.lift_arrived(),
            LiftState::Opening => true,
            LiftState::Waiting => true,
            LiftState::Closing => self.lift_closing(),
            LiftState::Checking => self.lift_checking(),
            LiftState::Checked => false,
            LiftState::Moving => self.lift_moving(),
        };
        if is_next {
            self.lift_state = self.lift_state.next();
        }
    }

    fn lift_arrived(&mut self) -> bool {
        self.set_floor_key(self.floor_state, false);
        self.update_direction();
        true
    }

    fn lift_closing(&mut self) -> bool {
        self.update_direction();
        if self.direction != Directions::None {
            self.lift_state = self.lift_state.next();
        }
        true
    }

    fn lift_checking(&mut self) -> bool {
        self.update_direction();
        self.direction != Directions::None
    }

    fn lift_moving(&mut self) -> bool {
        self.move_floor();
        self.has_current_by_key()
    }

    fn move_floor(&mut self) {
        match self.direction {
            Directions::Up => self.floor_state = self.floor_state.up(),
            Directions::Down => self.floor_state = self.floor_state.down(),
            Directions::None => todo!(),
        }
    }

    pub fn open(&mut self) {
        self.lift_state = self.lift_state.open();
    }

    pub fn close(&mut self) {
        self.lift_state = self.lift_state.close();
    }

    pub fn keys(&self) -> &[bool] {
        &self.floor_keys
    }
}
