use jni::JNIEnv;
use jni::objects::JString;
use jni::sys::{jint, jboolean};

use either::Either;

type JNIResult = Result<Either<String, Either<u64, bool>>, String>;

pub fn jni_transformer<T>(env: &mut JNIEnv, value: &T) -> JNIResult
where
    T: 'static,
{
    match std::any::TypeId::of::<T>() {
        t if t == std::any::TypeId::of::<JString<'static>>() => {
            let jstring = unsafe { std::mem::transmute_copy::<T, JString<'static>>(&value) };
            let rust_string: String = env.get_string(&jstring)
                .map_err(|e| format!("Failed to convert JString to Rust String: {:?}", e))?
                .into();
            Ok(Either::Left(rust_string))
        },
        t if t == std::any::TypeId::of::<jint>() => {
            let int_value = unsafe { std::mem::transmute_copy::<T, jint>(&value) };
            Ok(Either::Right(Either::Left(int_value as u64)))
        },
        t if t == std::any::TypeId::of::<jboolean>() => {
            let bool_value = unsafe { std::mem::transmute_copy::<T, jboolean>(&value) };
            Ok(Either::Right(Either::Right(bool_value != 0)))
        },
        _ => Err(format!("Unsupported type for jr_transform")),
    }
}

pub fn string_unwrap(data: JNIResult) -> String {
    match data {
        Ok(Either::Left(value)) => value,
        Ok(Either::Right(_)) => panic!("Expected left value, but got right value"),
        Err(e) => panic!("Error: {}", e),
    }
}

pub fn u64_unwrap(data: JNIResult) -> u64 {
    match data {
        Ok(Either::Left(_)) => panic!("Expected left value, but got right value"),
        Ok(Either::Right(Either::Left(value))) => value,
        Ok(Either::Right(Either::Right(_))) => panic!("Expected left value, but got right value"),
        Err(e) => panic!("Error: {}", e),
    }
}

pub fn bool_unwrap(data: JNIResult) -> bool {
    match data {
        Ok(Either::Left(_)) => panic!("Expected left value, but got right value"),
        Ok(Either::Right(Either::Left(_))) => panic!("Expected left value, but got right value"),
        Ok(Either::Right(Either::Right(value))) => value,
        Err(e) => panic!("Error: {}", e),
    }
}

