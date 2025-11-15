//-----------------------------------------------------------------------------//
// Rust Publish/Subscribe Pattern - Spare time development for fun             //
// (c) 2025 Laurent Lardinois https://be.linkedin.com/in/laurentlardinois      //
//                                                                             //
// https://github.com/type-one/PublishSubscribeRust                            //
//                                                                             //
// MIT License                                                                 //
//                                                                             //
// This software is provided 'as-is', without any express or implied           //
// warranty.In no event will the authors be held liable for any damages        //
// arising from the use of this software.                                      //
//                                                                             //
// Permission is granted to anyone to use this software for any purpose,       //
// including commercial applications, and to alter itand redistribute it       //
// freely, subject to the following restrictions :                             //
//                                                                             //
// 1. The origin of this software must not be misrepresented; you must not     //
// claim that you wrote the original software.If you use this software         //
// in a product, an acknowledgment in the product documentation would be       //
// appreciated but is not required.                                            //
// 2. Altered source versions must be plainly marked as such, and must not be  //
// misrepresented as being the original software.                              //
// 3. This notice may not be removed or altered from any source distribution.  //
//-----------------------------------------------------------------------------//

/// A simple finite state machine (FSM) example representing a traffic light system.
#[derive(Clone, Copy)]
enum TrafficLightState {
    Off,
    OperableInitializing,
    OperableRed(i32),
    OperableGreen(i32),
    OperableYellow(i32),
}

/// Events that trigger state transitions in the traffic light FSM.
enum TrafficLightEvent {
    PowerOn,
    PowerOff,
    InitDone,
    NextState,
}

/// Implementation of the FSM logic for the traffic light system.
impl TrafficLightState {
    /// Handles the PowerOn event.
    fn power_on(&self) -> Self {
        match *self {
            TrafficLightState::Off => TrafficLightState::OperableInitializing,
            _ => *self,
        }
    }

    /// Handles the InitDone event.
    fn init_done(&self) -> Self {
        match *self {
            TrafficLightState::OperableInitializing => TrafficLightState::OperableRed(10),
            _ => *self,
        }
    }

    /// Handles the NextState event.
    fn next_state(&self) -> Self {
        match *self {
            TrafficLightState::OperableRed(_) => TrafficLightState::OperableGreen(15),
            TrafficLightState::OperableGreen(_) => TrafficLightState::OperableYellow(3),
            TrafficLightState::OperableYellow(_) => TrafficLightState::OperableRed(10),
            _ => *self,
        }
    }

    /// Handles the PowerOff event.
    fn power_off(&self) -> Self {
        TrafficLightState::Off
    }

    /// Processes an event and returns the new state.
    fn on_event(&self, event: TrafficLightEvent) -> Self {
        match event {
            TrafficLightEvent::PowerOn => self.power_on(),
            TrafficLightEvent::InitDone => self.init_done(),
            TrafficLightEvent::NextState => self.next_state(),
            TrafficLightEvent::PowerOff => self.power_off(),
        }
    }

    /// Prints the current state of the traffic light.
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

/// Test function for the traffic light FSM.
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

/// Main function to run the FSM test.
pub fn fsm_test() {
    println!("Starting Finite State Machine test ...");
    println!("-----------------------------------------------");

    // Test sync queue
    test_fsm();

    println!("Finite State Machine test completed.");
    println!("-----------------------------------------------");
}
