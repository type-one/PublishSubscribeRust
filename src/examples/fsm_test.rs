// possible states
enum TrafficLightState {
    Off,
    Operable_Initializing,
    Operable_Red(i32),
    Operable_Green(i32),
    Operable_Yellow(i32),
}

enum TrafficLightEvent {
    Power_On,
    Power_Off,
    Init_Done,
    Next_State,
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
            (TrafficLightState::Off, TrafficLightEvent::Power_On) => {
                self.state = TrafficLightState::Operable_Initializing;
                println!("Transition to Initializing");
            }
            (TrafficLightState::Operable_Initializing, TrafficLightEvent::Init_Done) => {
                self.state = TrafficLightState::Operable_Red(10);
                println!("Transition to Red");
            }
            (TrafficLightState::Operable_Red(_), TrafficLightEvent::Next_State) => {
                self.state = TrafficLightState::Operable_Green(15);
                println!("Transition to Green");
            }
            (TrafficLightState::Operable_Green(_), TrafficLightEvent::Next_State) => {
                self.state = TrafficLightState::Operable_Yellow(3);
                println!("Transition to Yellow");
            }
            (TrafficLightState::Operable_Yellow(_), TrafficLightEvent::Next_State) => {
                self.state = TrafficLightState::Operable_Red(10);
                println!("Transition to Red");
            }
            (_, TrafficLightEvent::Power_Off) => {
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

    fsm.handle_event(TrafficLightEvent::Power_On);
    fsm.handle_event(TrafficLightEvent::Init_Done);
    fsm.handle_event(TrafficLightEvent::Next_State);
    fsm.handle_event(TrafficLightEvent::Next_State);
    fsm.handle_event(TrafficLightEvent::Next_State);
    fsm.handle_event(TrafficLightEvent::Next_State);
    fsm.handle_event(TrafficLightEvent::Power_Off);
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
