mod utils;

use solana_client::nonblocking::rpc_client::RpcClient;
use std::sync::{Arc, RwLock};

use jni::objects::{JClass, JString};
use jni::sys::{jboolean, jint, jstring};
use jni::JNIEnv;

use ore_lib::args::MineArgs;
use ore_lib::miner::Miner;
use ore_lib::Manager;

use crate::utils::{bool_unwrap, jni_transformer, string_unwrap, u64_unwrap};

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
    let keypair_filepath = string_unwrap(jni_transformer(&mut env, &keypair_filepath.clone()));
    let rpc_client = string_unwrap(jni_transformer(&mut env, &rpc_client.clone()));
    let priority_fee = u64_unwrap(jni_transformer(&mut env, &priority_fee));
    let dynamic_fee_url = string_unwrap(jni_transformer(&mut env, &dynamic_fee_url.clone()));
    let dynamic_fee = bool_unwrap(jni_transformer(&mut env, &dynamic_fee));
    let fee_payer_filepath = string_unwrap(jni_transformer(&mut env, &fee_payer_filepath.clone()));
    let jito_client = string_unwrap(jni_transformer(&mut env, &jito_client.clone()));
    let tip = u64_unwrap(jni_transformer(&mut env, &tip));
    let pool_url = string_unwrap(jni_transformer(&mut env, &pool_url.clone()));
    let cores = u64_unwrap(jni_transformer(&mut env, &cores));
    let buffer_time = u64_unwrap(jni_transformer(&mut env, &buffer_time));

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
