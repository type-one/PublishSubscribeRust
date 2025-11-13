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

use colored::Colorize;
use std::char::REPLACEMENT_CHARACTER;
use std::fmt::Display;
use std::sync::{Arc, Mutex};

use crate::tools::async_observer::AsyncObserver;
use crate::tools::data_task::DataTask;
use crate::tools::periodic_task::PeriodicTask;
use crate::tools::sync_dictionary::SyncDictionary;
use crate::tools::sync_observer::{LooseCoupledHandler, Observer, Subject, SubjectTrait};
use crate::tools::sync_queue::SyncQueue;
use crate::tools::task_trait::TaskTrait;
use crate::tools::worker_pool::WorkerPool;
use crate::tools::worker_task::WorkerTask;
use crate::tools::worker_trait::WorkerTrait;

/// Enum representing different types of data values.
#[derive(Debug, Clone)]
enum DataValue {
    Int(i32),
    Float(f64),
    Text(String),
}

/// Enum representing different types of events.
#[derive(Debug, Clone)]
enum Event {
    VariableUpdate(String, DataValue),
    Alert(String),
}

/// Enum representing different types of commands.
#[derive(Debug, Clone)]
enum Command {
    Start,
    Stop,
    Pause,
    Resume,
}

/// Implement Display trait for Command enum for better logging.
impl Display for Command {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Command::Start => write!(f, "Start"),
            Command::Stop => write!(f, "Stop"),
            Command::Pause => write!(f, "Pause"),
            Command::Resume => write!(f, "Resume"),
        }
    }
}

/// Struct representing a data hub.
struct DataHub {
    subject: Subject<String, DataValue>,
}

/// Implementation of SubjectTrait for DataHub.
impl SubjectTrait<String, DataValue> for DataHub {
    /// Subscribes an observer to a topic.
    fn subscribe(
        &mut self,
        topic: &String,
        observer: Arc<dyn Observer<String, DataValue> + Send + Sync>,
    ) {
        self.subject.subscribe(topic, observer);
    }

    /// Unsubscribes a single observer instance from a topic (keeps other observers for the same topic).
    fn unsubscribe(
        &mut self,
        topic: &String,
        observer: &Arc<dyn Observer<String, DataValue> + Send + Sync>,
    ) {
        self.subject.unsubscribe(topic, observer);
    }

    /// Publishes an event to a topic.
    fn publish(&self, topic: &String, event: &DataValue) {
        self.subject.publish(topic, event);
    }

    /// Subscribes a handler to a topic.
    fn subscribe_handler(
        &mut self,
        topic: &String,
        handler: Arc<LooseCoupledHandler<String, DataValue>>,
        handler_name: &str,
    ) {
        self.subject.subscribe_handler(topic, handler, handler_name);
    }

    /// Unsubscribes a handler from a topic (keeps other handlers for the same topic).
    fn unsubscribe_handler(&mut self, topic: &String, handler_name: &str) {
        self.subject.unsubscribe_handler(topic, handler_name);
    }

    /// Publishes an event to a topic with origin information.
    fn publish_named(&self, topic: &String, event: &DataValue, origin: &str) {
        self.subject.publish_named(topic, event, origin);
    }
}
/// Struct representing an events hub.
struct EventsHub {
    subject: Subject<String, Event>,
}

/// Implementation of SubjectTrait for EventsHub.
impl SubjectTrait<String, Event> for EventsHub {
    /// Subscribes an observer to a topic.
    fn subscribe(
        &mut self,
        topic: &String,
        observer: Arc<dyn Observer<String, Event> + Send + Sync>,
    ) {
        self.subject.subscribe(topic, observer);
    }

    /// Unsubscribes a single observer instance from a topic (keeps other observers for the same topic).
    fn unsubscribe(
        &mut self,
        topic: &String,
        observer: &Arc<dyn Observer<String, Event> + Send + Sync>,
    ) {
        self.subject.unsubscribe(topic, observer);
    }

    /// Publishes an event to a topic.
    fn publish(&self, topic: &String, event: &Event) {
        self.subject.publish(topic, event);
    }

    /// Subscribes a handler to a topic.
    fn subscribe_handler(
        &mut self,
        topic: &String,
        handler: Arc<LooseCoupledHandler<String, Event>>,
        handler_name: &str,
    ) {
        self.subject.subscribe_handler(topic, handler, handler_name);
    }

    /// Unsubscribes a handler from a topic (keeps other handlers for the same topic).
    fn unsubscribe_handler(&mut self, topic: &String, handler_name: &str) {
        self.subject.unsubscribe_handler(topic, handler_name);
    }

    /// Publishes an event to a topic with origin information.
    fn publish_named(&self, topic: &String, event: &Event, origin: &str) {
        self.subject.publish_named(topic, event, origin);
    }
}

/// Struct representing a commands hub.
struct CommandsHub {
    subject: Subject<String, Command>,
}

/// Implementation of SubjectTrait for CommandsHub.
impl SubjectTrait<String, Command> for CommandsHub {
    /// Subscribes an observer to a topic.
    fn subscribe(
        &mut self,
        topic: &String,
        observer: Arc<dyn Observer<String, Command> + Send + Sync>,
    ) {
        self.subject.subscribe(topic, observer);
    }

    /// Unsubscribes a single observer instance from a topic (keeps other observers for the same topic).
    fn unsubscribe(
        &mut self,
        topic: &String,
        observer: &Arc<dyn Observer<String, Command> + Send + Sync>,
    ) {
        self.subject.unsubscribe(topic, observer);
    }

    /// Publishes an event to a topic.
    fn publish(&self, topic: &String, event: &Command) {
        self.subject.publish(topic, event);
    }

    /// Subscribes a handler to a topic.
    fn subscribe_handler(
        &mut self,
        topic: &String,
        handler: Arc<LooseCoupledHandler<String, Command>>,
        handler_name: &str,
    ) {
        self.subject.subscribe_handler(topic, handler, handler_name);
    }

    /// Unsubscribes a handler from a topic (keeps other handlers for the same topic).
    fn unsubscribe_handler(&mut self, topic: &String, handler_name: &str) {
        self.subject.unsubscribe_handler(topic, handler_name);
    }

    /// Publishes an event to a topic with origin information.
    fn publish_named(&self, topic: &String, event: &Command, origin: &str) {
        self.subject.publish_named(topic, event, origin);
    }
}

/// Struct representing the shared context.
struct Context {
    variables: SyncDictionary<String, DataValue>,
    data_archives: SyncQueue<DataValue>,
    events_archives: SyncQueue<Event>,
    data_hub: DataHub,
    events_hub: EventsHub,
    commands_hub: CommandsHub,
}

type ContextWrapper = Mutex<Context>;

/// Struct representing the classifier.
struct Classifier {
    worker_pool: Arc<Mutex<WorkerPool<ContextWrapper>>>,
    periodic_task: PeriodicTask<ContextWrapper>,
    data_observer: Arc<AsyncObserver<String, DataValue>>,
    events_observer: Arc<AsyncObserver<String, Event>>,
    commands_observer: Arc<AsyncObserver<String, Command>>,
}

/// Implementation of Classifier.
impl Classifier {
    pub fn new(context: Arc<ContextWrapper>) -> Self {
        let local_worker_pool = Arc::new(Mutex::new(WorkerPool::new(context.clone())));
        let local_data_observer = Arc::new(AsyncObserver::<String, DataValue>::new());
        let local_events_observer = Arc::new(AsyncObserver::<String, Event>::new());
        let local_commands_observer = Arc::new(AsyncObserver::<String, Command>::new());

        let local_worker_pool_clone = local_worker_pool.clone();
        let local_data_observer_clone = local_data_observer.clone();
        let local_events_observer_clone = local_events_observer.clone();
        let local_commands_observer_clone = local_commands_observer.clone();

        Classifier {
            worker_pool: local_worker_pool,
            periodic_task: PeriodicTask::new(
                context,
                "ClassifierPeriodicTask".to_string(),
                Arc::new(move |ctx: Arc<ContextWrapper>, task_name: &String| {
                    println!(
                        "{} '{}' {}",
                        "Classifier task".blue(),
                        task_name.blue(),
                        "is classifying data...".blue()
                    );

                    // Classification logic here

                    local_data_observer_clone.wait_for_events(1000);

                    if local_data_observer_clone.has_events() {
                        let events = local_data_observer_clone.pop_all_events();
                        for (topic, event, origin) in events {
                            println!(
                                "{} '{}' {} '{}' : data: {:?}",
                                "Classifier processing data on topic".blue(),
                                topic.blue(),
                                "origin".blue(),
                                origin.blue(),
                                event
                            );
                            // Process data event

                            // Update context variables based on classification
                            ctx.lock()
                                .unwrap()
                                .variables
                                .insert(topic.clone(), event.clone());

                            ctx.lock().unwrap().events_hub.publish_named(
                                &"SystemEvents".to_string(),
                                &Event::VariableUpdate(topic.clone(), event.clone()),
                                "Classifier",
                            );
                        }
                    }

                    if local_events_observer_clone.has_events() {
                        let events = local_events_observer_clone.pop_all_events();
                        for (topic, event, origin) in events {
                            println!(
                                "{} '{}' {} '{}' : event: {:?}",
                                "Classifier processing event on topic".blue(),
                                topic.blue(),
                                "origin".blue(),
                                origin.blue(),
                                event
                            );
                            // Process event
                        }
                    }

                    if local_commands_observer_clone.has_events() {
                        let events = local_commands_observer_clone.pop_all_events();
                        for (topic, command, origin) in events {
                            println!(
                                "{} '{}' {} '{}' : command: {:?}",
                                "Classifier received command on topic".blue(),
                                topic.blue(),
                                "origin".blue(),
                                origin.blue(),
                                command
                            );

                            // Process command in worker pool
                            local_worker_pool_clone.lock().unwrap().delegate(Arc::new(
                                move |ctx: Arc<ContextWrapper>, task_name: &String| {
                                    println!(
                                        "{} '{}' {} '{}'",
                                        "Worker in task".bright_blue(),
                                        task_name.bright_blue(),
                                        "executing command".bright_blue(),
                                        command.to_string().bright_blue()
                                    );

                                    // Worker processing logic here

                                    match command {
                                        Command::Start => {
                                            println!(
                                                "{}",
                                                "Starting classifier operations...".bright_blue()
                                            );
                                        }
                                        Command::Stop => {
                                            println!(
                                                "{}",
                                                "Stopping classifier operations...".bright_blue()
                                            );
                                        }
                                        Command::Pause => {
                                            println!(
                                                "{}",
                                                "Pausing classifier operations...".bright_blue()
                                            );
                                        }
                                        Command::Resume => {
                                            println!(
                                                "{}",
                                                "Resuming classifier operations...".bright_blue()
                                            );
                                        }
                                    }
                                },
                            ));
                        }
                    }
                }),
                1500,
            ),
            data_observer: local_data_observer,
            events_observer: local_events_observer,
            commands_observer: local_commands_observer,
        }
    }

    pub fn start(&mut self) {
        self.worker_pool.lock().unwrap().start();
        self.periodic_task.start();
        println!("{}", "Classifier started.".blue());
    }
}

/// Implementation of Drop for Classifier.
impl Drop for Classifier {
    fn drop(&mut self) {
        // Clean up resources when the Classifier is dropped
        println!("{}", "Classifier stopped.".blue());
    }
}

/// Struct representing the archiver.
struct Archiver {
    worker_task: Arc<Mutex<WorkerTask<ContextWrapper>>>,
    periodic_task: PeriodicTask<ContextWrapper>,
    data_observer: Arc<AsyncObserver<String, DataValue>>,
    events_observer: Arc<AsyncObserver<String, Event>>,
    commands_observer: Arc<AsyncObserver<String, Command>>,
}

/// Implementation of Archiver.
impl Archiver {
    pub fn new(context: Arc<ContextWrapper>) -> Self {
        let local_worker_task = Arc::new(Mutex::new(WorkerTask::new(
            context.clone(),
            "ArchiverWorkerTask".to_string(),
        )));
        let local_data_observer = Arc::new(AsyncObserver::<String, DataValue>::new());
        let local_events_observer = Arc::new(AsyncObserver::<String, Event>::new());
        let local_commands_observer = Arc::new(AsyncObserver::<String, Command>::new());

        let local_worker_task_clone = local_worker_task.clone();
        let local_data_observer_clone = local_data_observer.clone();
        let local_events_observer_clone = local_events_observer.clone();
        let local_commands_observer_clone = local_commands_observer.clone();

        Archiver {
            worker_task: Arc::new(Mutex::new(WorkerTask::new(
                context.clone(),
                "ArchiverWorkerTask".to_string(),
            ))),
            periodic_task: PeriodicTask::new(
                context,
                "ArchiverPeriodicTask".to_string(),
                Arc::new(move |ctx: Arc<ContextWrapper>, task_name: &String| {
                    println!(
                        "{} '{}' {}",
                        "Archiver task".green(),
                        task_name.green(),
                        "is archiving data...".green()
                    );

                    // Archiving logic here

                    local_data_observer_clone.wait_for_events(1000);

                    if local_data_observer_clone.has_events() {
                        let events = local_data_observer_clone.pop_all_events();
                        for (topic, data, origin) in events {
                            println!(
                                "{} '{}' {} '{}' : data: {:?}",
                                "Archiver processing data on topic".green(),
                                topic.green(),
                                "origin".green(),
                                origin.green(),
                                data
                            );
                            // Archive data
                            ctx.lock().unwrap().data_archives.enqueue(data);
                        }
                    }

                    if local_events_observer_clone.has_events() {
                        let events = local_events_observer_clone.pop_all_events();
                        for (topic, event, origin) in events {
                            println!(
                                "{} '{}' {} '{}' : event: {:?}",
                                "Archiver processing event on topic".green(),
                                topic.green(),
                                "origin".green(),
                                origin.green(),
                                event
                            );
                            // Archive event
                            ctx.lock().unwrap().events_archives.enqueue(event);
                        }
                    }

                    if local_commands_observer_clone.has_events() {
                        let events = local_commands_observer_clone.pop_all_events();
                        for (topic, command, origin) in events {
                            println!(
                                "{} '{}' {} '{}' : command: {:?}",
                                "Archiver received command on topic".green(),
                                topic.green(),
                                "origin".green(),
                                origin.green(),
                                command
                            );

                            // Process command in worker task
                            local_worker_task_clone.lock().unwrap().delegate(Arc::new(
                                move |ctx: Arc<ContextWrapper>, task_name: &String| {
                                    println!(
                                        "{} '{}' {} '{}'",
                                        "Worker in task".yellow(),
                                        task_name.yellow(),
                                        "executing command".yellow(),
                                        command.to_string().yellow()
                                    );

                                    // Worker processing logic here

                                    match command {
                                        Command::Start => {
                                            println!(
                                                "{}",
                                                "Starting archive operations...".yellow()
                                            );
                                        }
                                        Command::Stop => {
                                            println!(
                                                "{}",
                                                "Stopping archive operations...".yellow()
                                            );
                                        }
                                        Command::Pause => {
                                            println!(
                                                "{}",
                                                "Pausing archive operations...".yellow()
                                            );
                                        }
                                        Command::Resume => {
                                            println!(
                                                "{}",
                                                "Resuming archive operations...".yellow()
                                            );
                                        }
                                    }
                                },
                            ));
                        }
                    }
                }),
                2000,
            ),
            data_observer: local_data_observer,
            events_observer: local_events_observer,
            commands_observer: local_commands_observer,
        }
    }

    /// Starts the archiver.
    pub fn start(&mut self) {
        self.worker_task.lock().unwrap().start();
        self.periodic_task.start();
        println!("{}", "Archiver started.".green());
    }
}

/// Implementation of Drop for Archiver.
impl Drop for Archiver {
    fn drop(&mut self) {
        // Clean up resources when the Archiver is dropped
        println!("{}", "Archiver stopped.".green());
    }
}

/// Struct representing the emitter.
struct Emitter {
    periodic_task: PeriodicTask<ContextWrapper>,
    relative_time: Arc<Mutex<f64>>, // used via clone by internal periodic task
}

/// Implementation of Emitter.
impl Emitter {
    pub fn new(context: Arc<ContextWrapper>) -> Self {
        let rel_time = Arc::new(Mutex::new(0.0));
        let rel_time_clone = rel_time.clone();

        Emitter {
            periodic_task: PeriodicTask::new(
                context,
                "EmitterTask".to_string(),
                Arc::new(move |ctx: Arc<ContextWrapper>, task_name: &String| {
                    println!(
                        "{} {} {}",
                        "Emitter task".purple(),
                        task_name.purple(),
                        "is emitting data...".purple()
                    );

                    // Emit data logic here
                    let t = *rel_time.lock().unwrap();
                    const SINE_WAVE_FREQUENCY_HZ: f64 = 1.0; // 1 Hz
                    ctx.lock().unwrap().data_hub.publish_named(
                        &"SensorData".to_string(),
                        &DataValue::Float(f64::sin(
                            std::f64::consts::PI * 2.0 * SINE_WAVE_FREQUENCY_HZ * t,
                        )),
                        "Emitter",
                    );
                    *rel_time.lock().unwrap() += 1.0; // assuming period is 1000 ms
                }),
                1000,
            ),
            relative_time: rel_time_clone,
        }
    }

    pub fn start(&mut self) {
        self.periodic_task.start();
        println!("{}", "Emitter started.".purple());
    }
}

/// Implementation of Drop for Emitter.
impl Drop for Emitter {
    fn drop(&mut self) {
        // Clean up resources when the Emitter is dropped
        println!("{}", "Emitter stopped.".purple());
    }
}

/// Advanced test function for Publish/Subscribe with parsing.
pub fn advanced_test() {
    println!(
        "{}",
        "Advanced test with Publish/Subscribe and parsing...".white()
    );
    println!(
        "{}",
        "---------------------------------------------------".white()
    );

    // Create shared context
    let context = Arc::new(Mutex::new(Context {
        variables: SyncDictionary::new(),
        data_archives: SyncQueue::new(),
        events_archives: SyncQueue::new(),
        data_hub: DataHub {
            subject: Subject::new("DataHub"),
        },
        events_hub: EventsHub {
            subject: Subject::new("EventsHub"),
        },
        commands_hub: CommandsHub {
            subject: Subject::new("CommandsHub"),
        },
    }));

    // Create components
    let mut emitter = Emitter::new(context.clone());
    let mut archiver = Archiver::new(context.clone());
    let mut classifier = Classifier::new(context.clone());

    // Setup subscriptions
    context
        .lock()
        .unwrap()
        .data_hub
        .subscribe(&"SensorData".to_string(), classifier.data_observer.clone());

    context.lock().unwrap().events_hub.subscribe(
        &"SystemEvents".to_string(),
        classifier.events_observer.clone(),
    );

    context.lock().unwrap().commands_hub.subscribe(
        &"ControlCommands".to_string(),
        classifier.commands_observer.clone(),
    );

    context
        .lock()
        .unwrap()
        .data_hub
        .subscribe(&"SensorData".to_string(), archiver.data_observer.clone());

    context.lock().unwrap().events_hub.subscribe(
        &"SystemEvents".to_string(),
        archiver.events_observer.clone(),
    );

    context.lock().unwrap().commands_hub.subscribe(
        &"ControlCommands".to_string(),
        archiver.commands_observer.clone(),
    );

    // Setup loose-coupled handlers for spying
    context.lock().unwrap().data_hub.subscribe_handler(
        &"SensorData".to_string(),
        Arc::new(|topic: &String, event: &DataValue, origin: &str| {
            println!(
                "\x1B[38;5;43mSpy handler received data on topic '{}', origin '{}', event: {:?}",
                topic, origin, event
            );
        }),
        "SpyDataHandler",
    );

    context.lock().unwrap().events_hub.subscribe_handler(
        &"SystemEvents".to_string(),
        Arc::new(|topic: &String, event: &Event, origin: &str| {
            println!(
                "\x1B[38;5;43mSpy handler received event on topic '{}', origin '{}', event: {:?}",
                topic, origin, event
            );
        }),
        "SpyEventsHandler",
    );

    context.lock().unwrap().commands_hub.subscribe_handler(
        &"ControlCommands".to_string(),
        Arc::new(|topic: &String, event: &Command, origin: &str| {
            println!(
                "\x1B[38;5;43mSpy handler received command on topic '{}', origin '{}', command: {}",
                topic, origin, event
            );
        }),
        "SpyCommandsHandler",
    );

    // Start components
    classifier.start();
    archiver.start();
    emitter.start();

    std::thread::sleep(std::time::Duration::from_secs(5));

    // Simulate publishing commands
    println!("{}", "Publish start command...".white());
    context.lock().unwrap().commands_hub.publish_named(
        &"ControlCommands".to_string(),
        &Command::Start,
        "AdvancedTest",
    );

    std::thread::sleep(std::time::Duration::from_secs(1));

    println!("{}", "Publish pause command...".white());
    context.lock().unwrap().commands_hub.publish_named(
        &"ControlCommands".to_string(),
        &Command::Pause,
        "AdvancedTest",
    );

    std::thread::sleep(std::time::Duration::from_secs(1));

    println!("{}", "Publish resume command...".white());
    context.lock().unwrap().commands_hub.publish_named(
        &"ControlCommands".to_string(),
        &Command::Resume,
        "AdvancedTest",
    );

    std::thread::sleep(std::time::Duration::from_secs(1));

    println!("{}", "Publish stop command...".white());
    context.lock().unwrap().commands_hub.publish_named(
        &"ControlCommands".to_string(),
        &Command::Stop,
        "AdvancedTest",
    );

    // Let the system run for a while
    println!("{}", "Let the system run for a while...".white());

    std::thread::sleep(std::time::Duration::from_secs(10));

    // Simulate publishing an alert event
    println!("{}", "Publish alert event...".white());
    context.lock().unwrap().events_hub.publish_named(
        &"SystemEvents".to_string(),
        &Event::Alert("High temperature detected!".to_string()),
        "AdvancedTest",
    );

    std::thread::sleep(std::time::Duration::from_secs(10));

    println!(
        "{}",
        "------------------- Advanced Test Completed -------------------".white()
    );
}
