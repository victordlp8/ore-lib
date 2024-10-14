use jni::JNIEnv;
use jni::objects::JString;
use jni::sys::{jint, jboolean};

use either::Either;

type JNIResult = Result<Either<String, Either<u64, bool>>, String>;

pub fn string_unwrap(env: &mut JNIEnv, data: JString) -> String {
    env.get_string(&data)
        .expect("Failed to convert JString to Rust String")
        .into()
}
