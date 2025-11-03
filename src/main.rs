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

use publish_subscribe_rs::tools::sync_dictionary;
use publish_subscribe_rs::tools::sync_object;
use publish_subscribe_rs::tools::sync_queue;

fn main() {
    let sync_queue = sync_queue::SyncQueue::<i32>::new();
    sync_queue.enqueue(10);
    let item = sync_queue.dequeue();
    println!("Dequeued item: {:?}", item);

    let mut sync = sync_object::SyncObject::new(false);
    //sync.wait_for_signal();

    sync.wait_for_signal_timeout(1000);

    let dict = sync_dictionary::SyncDictionary::<String, i32>::new();
    dict.insert("key1".to_string(), 42);
    if let Some(value) = dict.get(&"key1".to_string()) {
        println!("Value for 'key1': {}", value);
    }
}
