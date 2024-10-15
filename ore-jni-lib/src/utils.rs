use jni::JNIEnv;
use jni::objects::JString;

pub fn string_unwrap(env: &mut JNIEnv, data: JString) -> String {
    env.get_string(&data)
        .expect("Failed to convert JString to Rust String")
        .into()
}
