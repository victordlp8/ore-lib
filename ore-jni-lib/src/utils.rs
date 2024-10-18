use jni::JNIEnv;
use jni::objects::JString;

pub fn string_unwrap(env: &mut JNIEnv, data: JString) -> String {
    env.get_string(&data)
        .expect("Failed to convert JString to Rust String")
        .into()
}

pub fn throw_java_exception(env: &mut JNIEnv, exception_class: &str, message: &str) {
    let exception_class = env.find_class(exception_class).unwrap();
    env.throw_new(exception_class, message).unwrap();
}
