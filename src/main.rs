//-----------------------------------------------------------------------------//
// Rust Publish/Subscribe Pattern - Spare time development for fun             //
// (c) 2025-2026 Laurent Lardinois https://be.linkedin.com/in/laurentlardinois //
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

use pubsub_rs::examples::advanced_test;
use pubsub_rs::examples::basic_test;
use pubsub_rs::examples::cjson_test;
use pubsub_rs::examples::fsm_test;
use pubsub_rs::examples::json_test;

// Prevent heap fragmentation from frequent small event/message allocations by
// caching and reusing small power-of-two blocks. Opt-in via `--features pool_allocator`.
#[cfg(feature = "pool_allocator")]
#[global_allocator]
static GLOBAL_ALLOCATOR: pubsub_rs::tools::pool_allocator::PoolAllocator =
    pubsub_rs::tools::pool_allocator::PoolAllocator::new();

/// Main entry point for testing the synchronization tools.
fn main() {
    // all-in-one basic tests of the different helper tools
    basic_test::basic_test();

    // specific publish/subscribe advanced test with parsing
    advanced_test::advanced_test();

    // finite state machine test
    fsm_test::fsm_test();

    // CJSON test (simple C bindings)
    cjson_test::cjson_test();

    // JSON test
    json_test::json_test();
}
