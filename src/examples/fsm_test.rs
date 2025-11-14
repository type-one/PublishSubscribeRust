// possible states
#[derive(Clone, Copy)]
enum TrafficLightState {
    Off,
    OperableInitializing,
    OperableRed(i32),
    OperableGreen(i32),
    OperableYellow(i32),
}

enum TrafficLightEvent {
    PowerOn,
    PowerOff,
    InitDone,
    NextState,
}
impl TrafficLightState {
    fn power_on(&self) -> Self {
        match *self {
            TrafficLightState::Off => TrafficLightState::OperableInitializing,
            _ => *self,
        }
    }

    fn init_done(&self) -> Self {
        match *self {
            TrafficLightState::OperableInitializing => TrafficLightState::OperableRed(10),
            _ => *self,
        }
    }

    fn next_state(&self) -> Self {
        match *self {
            TrafficLightState::OperableRed(_) => TrafficLightState::OperableGreen(15),
            TrafficLightState::OperableGreen(_) => TrafficLightState::OperableYellow(3),
            TrafficLightState::OperableYellow(_) => TrafficLightState::OperableRed(10),
            _ => *self,
        }
    }

    fn power_off(&self) -> Self {
        TrafficLightState::Off
    }

    fn on_event(&self, event: TrafficLightEvent) -> Self {
        match event {
            TrafficLightEvent::PowerOn => self.power_on(),
            TrafficLightEvent::InitDone => self.init_done(),
            TrafficLightEvent::NextState => self.next_state(),
            TrafficLightEvent::PowerOff => self.power_off(),
        }
    }

    fn on_state(&self) {
        match *self {
            TrafficLightState::Off => println!("State: Off"),
            TrafficLightState::OperableInitializing => println!("State: Initializing"),
            TrafficLightState::OperableRed(duration) => {
                println!("State: Red for {} seconds", duration)
            }
            TrafficLightState::OperableGreen(duration) => {
                println!("State: Green for {} seconds", duration)
            }
            TrafficLightState::OperableYellow(duration) => {
                println!("State: Yellow for {} seconds", duration)
            }
        }
    }
}

fn test_fsm() {
    let mut state = TrafficLightState::Off;

    state.on_state();
    state = state.on_event(TrafficLightEvent::PowerOn);
    state.on_state();
    state = state.on_event(TrafficLightEvent::InitDone);
    state.on_state();
    state = state.on_event(TrafficLightEvent::NextState);
    state.on_state();
    state = state.on_event(TrafficLightEvent::NextState);
    state.on_state();
    state = state.on_event(TrafficLightEvent::NextState);
    state.on_state();
    state = state.on_event(TrafficLightEvent::NextState);
    state.on_state();
    state = state.on_event(TrafficLightEvent::PowerOff);
    state.on_state();
}

/// The main basic test function that calls all individual tests.
pub fn fsm_test() {
    println!("Starting Finite State Machine test ...");
    println!("-----------------------------------------------");

    // Test sync queue
    test_fsm();

    println!("Finite State Machine test completed.");
    println!("-----------------------------------------------");
}
