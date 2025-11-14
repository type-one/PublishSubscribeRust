// possible states
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

struct TrafficLightFSM {
    state: TrafficLightState,
}

impl TrafficLightFSM {
    fn new() -> Self {
        TrafficLightFSM {
            state: TrafficLightState::Off,
        }
    }

    fn handle_event(&mut self, event: TrafficLightEvent) {
        match (&self.state, event) {
            (TrafficLightState::Off, TrafficLightEvent::PowerOn) => {
                self.state = TrafficLightState::OperableInitializing;
                println!("Transition to Initializing");
            }
            (TrafficLightState::OperableInitializing, TrafficLightEvent::InitDone) => {
                self.state = TrafficLightState::OperableRed(10);
                println!("Transition to Red");
            }
            (TrafficLightState::OperableRed(_), TrafficLightEvent::NextState) => {
                self.state = TrafficLightState::OperableGreen(15);
                println!("Transition to Green");
            }
            (TrafficLightState::OperableGreen(_), TrafficLightEvent::NextState) => {
                self.state = TrafficLightState::OperableYellow(3);
                println!("Transition to Yellow");
            }
            (TrafficLightState::OperableYellow(_), TrafficLightEvent::NextState) => {
                self.state = TrafficLightState::OperableRed(10);
                println!("Transition to Red");
            }
            (_, TrafficLightEvent::PowerOff) => {
                self.state = TrafficLightState::Off;
                println!("Transition to Off");
            }
            _ => {
                println!("No transition for this event in current state");
            }
        }
    }
}

fn test_fsm() {
    let mut fsm = TrafficLightFSM::new();

    fsm.handle_event(TrafficLightEvent::PowerOn);
    fsm.handle_event(TrafficLightEvent::InitDone);
    fsm.handle_event(TrafficLightEvent::NextState);
    fsm.handle_event(TrafficLightEvent::NextState);
    fsm.handle_event(TrafficLightEvent::NextState);
    fsm.handle_event(TrafficLightEvent::NextState);
    fsm.handle_event(TrafficLightEvent::PowerOff);
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
