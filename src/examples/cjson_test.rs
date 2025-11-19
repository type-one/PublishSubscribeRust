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

// https://docs.rs/cc/latest/cc/

// A simple example for testing the CJSON C library bindings in Rust.

unsafe extern "C" {
    // extern functions if needed
    fn cJSON_Version() -> *const i8;
}

/// Test function for the CJSON C library.
fn test_cjson() {
    unsafe {
        let version_ptr = cJSON_Version();
        let c_str = std::ffi::CStr::from_ptr(version_ptr);
        let version_str = c_str.to_str().unwrap();
        println!("CJSON Version: {}", version_str);
    }
}

/// Main function to run the CJSON C library test.
pub fn cjson_test() {
    println!("Starting CJSON test ...");
    println!("-----------------------------------------------");

    // Test CJSON functionality
    test_cjson();

    println!("CJSON test completed.");
    println!("-----------------------------------------------");
}
