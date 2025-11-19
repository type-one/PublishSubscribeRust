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

// https://docs.rs/serde_json/latest/serde_json/

// A simple example for testing the JSON Serde library in Rust.

/// Test function for the JSON Serde library.
fn test_json() {
    // A sample JSON string
    let data = r#"
        {
            "name": "Julius Caesar",
            "age": 42,
            "phones": [
                "+39 6234567",
                "+39 6345678"
            ]
        }"#;

    // Parse the string of data into a serde_json::Value.
    let v: serde_json::Value = serde_json::from_str(data).unwrap();

    // Access parts of the data by indexing with square brackets.
    println!("Please call {} at the number {}", v["name"], v["phones"][0]);
}

/// Main function to run the JSON Serde library test.
pub fn json_test() {
    println!("Starting JSON test ...");
    println!("-----------------------------------------------");

    // Test JSON functionality
    test_json();

    println!("JSON test completed.");
    println!("-----------------------------------------------");
}
