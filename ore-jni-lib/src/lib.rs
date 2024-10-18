mod utils;

use solana_client::nonblocking::rpc_client::RpcClient;
use tokio::runtime::Runtime;
use std::sync::{Arc, RwLock};
use std::panic::{self, AssertUnwindSafe};

use jni::objects::{JClass, JString};
use jni::sys::{jboolean, jint, jstring};
use jni::JNIEnv;

use ore_lib::args::MineArgs;
use ore_lib::miner::Miner;
use ore_lib::Manager;

use crate::utils::{string_unwrap, throw_java_exception};

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

    let rt = Runtime::new().expect("Failed to create Tokio runtime");

    let greeting = rt.block_on(async move {
        tokio::time::sleep(std::time::Duration::from_secs(2)).await;
        format!("Hello, I am {} the happy rustacean!", name)
    });

    env.new_string(greeting)
        .expect("Couldn't create Java string!")
        .into_raw()
}

#[no_mangle]
pub extern "system" fn Java_industries_dlp8_rust_OreJNILib_startMining(
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

    let rpc_client = Arc::new(RpcClient::new(rpc_client));
    let jito_client = Arc::new(RpcClient::new(jito_client));
    let tip = Arc::new(RwLock::new(tip));

    let miner = Miner::new(
        rpc_client,
        Some(priority_fee),
        Some(keypair_filepath),
        Some(dynamic_fee_url),
        dynamic_fee,
        Some(fee_payer_filepath),
        jito_client,
        tip,
    );

    let mining_args = MineArgs {
        pool_url: Some(pool_url),
        cores,
        buffer_time,
        boost_1: None,
        boost_2: None,
        boost_3: None,
    };

    let manager = Manager::new(miner, mining_args);
    Manager::set_global_manager(manager);
    let global_manager = Manager::get_global_manager();

    let result = tokio::runtime::Runtime::new().unwrap().block_on(async {
        let mut manager = global_manager.lock().await;
        match manager.start_mining() {
            Ok(_) => {
                println!("ore-jni-lib: Mining started");
                0
            }
            Err(e) => {
                let error_msg = format!("Error starting mining: {:?}", e);
                throw_java_exception(&mut env, "java/lang/RuntimeException", &error_msg);
                -1
            }
        }
    });

    result
}

#[no_mangle]
pub extern "system" fn Java_industries_dlp8_rust_OreJNILib_stopMining(
    mut env: JNIEnv,
    _class: JClass,
) -> jint {
    let result = panic::catch_unwind(AssertUnwindSafe(|| {
        let global_manager = Manager::get_global_manager();

        tokio::runtime::Runtime::new().unwrap().block_on(async {
            let manager = global_manager.lock().await;
            manager.stop_mining()
        })
    }));

    match result {
        Ok(Ok(_)) => {
            println!("ore-jni-lib: Mining stopped");
            0
        }
        Ok(Err(e)) => {
            let error_msg = format!("Error stopping mining: {:?}", e);
            eprintln!("{}", error_msg);
            throw_java_exception(&mut env, "java/lang/RuntimeException", &error_msg);
            -1
        }
        Err(panic_error) => {
            let error_msg = if let Some(s) = panic_error.downcast_ref::<String>() {
                s.clone()
            } else if let Some(s) = panic_error.downcast_ref::<&str>() {
                s.to_string()
            } else {
                "Unknown panic occurred".to_string()
            };
            eprintln!("Panic occurred: {}", error_msg);
            throw_java_exception(&mut env, "java/lang/RuntimeException", &error_msg);
            -1
        }
    }
}
