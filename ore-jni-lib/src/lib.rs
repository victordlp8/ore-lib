mod utils;

use solana_client::nonblocking::rpc_client::RpcClient;
use std::sync::{Arc, RwLock};

use jni::objects::{JClass, JString};
use jni::sys::{jboolean, jint, jstring};
use jni::JNIEnv;

use ore_lib::args::MineArgs;
use ore_lib::miner::Miner;
use ore_lib::Manager;

use crate::utils::string_unwrap;

// #[no_mangle]
// pub extern "system" fn JNI_OnLoad(vm: JavaVM, _reserved: *mut c_void) -> jint {
//     let mut env = vm.get_env().expect("Failed to get JNI env");

//     // Attempt to find the class using different package names
//     let class_names = [
//         "OreJNILib",
//         "industries/dlp8/rust/OreJNILib",
//         "com/example/OreJNILib",
//     ];

//     for class_name in class_names.iter() {
//         if let Ok(class) = env.find_class(class_name) {
//             let method = NativeMethod {
//                 name: "helloRust".into(),
//                 sig: "(Ljava/lang/String;)Ljava/lang/String;".into(),
//                 fn_ptr: helloRust as *mut c_void,
//             };

//             if env.register_native_methods(class, &[method]).is_ok() {
//                 println!("Successfully registered native method for class: {}", class_name);
//                 return JNI_VERSION_1_6;
//             }
//         }
//     }

//     panic!("Failed to find and register the Java class");
// }

#[no_mangle]
pub extern "system" fn Java_industries_dlp8_rust_OreJNILib_helloRust(
    mut env: JNIEnv,
    _class: JClass,
    name: JString,
) -> jstring {
    let name: String = env
        .get_string(&name)
        .expect("Couldn't get Java string!")
        .into();
    let greeting = format!("Hello, I am {} the happy rustacean!", name);
    env.new_string(greeting)
        .expect("Couldn't create Java string!")
        .into_raw()
}

#[no_mangle]
pub extern "system" fn Java_industries_dlp8_rust_OreJNILib_startPoolMining(
    mut env: JNIEnv,
    _class: JClass,
    keypair_filepath: JString,
    rpc_client: JString,
    priority_fee: jint,
    dynamic_fee_url: JString,
    dynamic_fee: jboolean,
    fee_payer_filepath: JString,
    jito_client: JString,
    tip: jint,
    pool_url: JString,
    cores: jint,
    buffer_time: jint,
) -> jint {
    let keypair_filepath = string_unwrap(&mut env, keypair_filepath);
    let rpc_client = string_unwrap(&mut env, rpc_client);
    let priority_fee = priority_fee as u64;
    let dynamic_fee_url = string_unwrap(&mut env, dynamic_fee_url);
    let dynamic_fee = dynamic_fee != 0;
    let fee_payer_filepath = string_unwrap(&mut env, fee_payer_filepath);
    let jito_client = string_unwrap(&mut env, jito_client);
    let tip = tip as u64;
    let pool_url = string_unwrap(&mut env, pool_url);
    let cores = cores as u64;
    let buffer_time = buffer_time as u64;

    let _miner = Miner::new(
        Arc::new(RpcClient::new(rpc_client)),
        Some(priority_fee),
        Some(keypair_filepath),
        Some(dynamic_fee_url),
        dynamic_fee,
        Some(fee_payer_filepath),
        Arc::new(RpcClient::new(jito_client)),
        Arc::new(RwLock::new(tip)),
    );

    let _mining_args = MineArgs {
        pool_url: Some(pool_url.clone()),
        cores: cores,
        buffer_time: buffer_time,
        boost_1: None,
        boost_2: None,
        boost_3: None,
    };

    let manager = Manager::new(_miner, _mining_args);

    // Since we can't make this function async, we'll use tokio's runtime to run the async code
    let rt = tokio::runtime::Runtime::new().unwrap();
    rt.block_on(async {
        match manager.mine().await {
            Ok(_) => {
                // Mining completed successfully
                println!("Mining operation completed successfully");
                0 // Return success code
            },
            Err(e) => {
                // An error occurred during mining
                eprintln!("Error during mining: {:?}", e);
                -1 // Return an error code
            }
        }
    })
}
