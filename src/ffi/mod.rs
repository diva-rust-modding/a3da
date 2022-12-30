use interoptopus::{
    extra_type, ffi_function, ffi_service, ffi_service_ctor, ffi_service_method, ffi_type,
    function, pattern,
    patterns::{api_guard::APIVersion, result::FFIError, slice::FFISlice, string::AsciiPointer},
    Inventory, InventoryBuilder,
};

#[ffi_type]
#[repr(C)]
#[derive(Default)]
pub struct Auth3d {
    // pub camera_root: FFISlice<()>,
    pub play_control: PlayControl,
    pub metadata: Metadata,
}

#[ffi_type]
#[repr(C)]
#[derive(Default)]
pub struct PlayControl {
    pub begin: u32,
    pub fps: u32,
    pub size: u32,
}

#[ffi_type]
#[repr(C)]
#[derive(Default)]
pub struct Metadata {
    pub converter_version: u32,
    pub property_version: u32,
    pub file_name: AsciiPointer<'static>,
}

#[ffi_type(patterns(ffi_error))]
#[repr(C)]
#[derive(Debug)]
pub enum FfiError {
    Ok = 0,
    NullPassed = 1,
    Panic = 2,
}

impl FFIError for FfiError {
    const SUCCESS: Self = Self::Ok;

    const NULL: Self = Self::NullPassed;

    const PANIC: Self = Self::Panic;
}

impl From<super::A3da> for Auth3d {
    fn from(value: super::A3da) -> Self {
        let super::A3da {
            camera_root,
            play_control,
            metadata,
        } = value;
        let play_control = play_control.into();
        let metadata = Default::default();
        Self {
            play_control,
            metadata,
        }
    }
}

impl From<super::PlayControl> for PlayControl {
    fn from(value: super::PlayControl) -> Self {
        let super::PlayControl { begin, fps, size } = value;
        Self { begin, fps, size }
    }
}

impl From<super::Metadata> for Metadata {
    fn from(value: super::Metadata) -> Self {
        let super::Metadata {
            converter,
            property,
            file_name,
        } = value;
        let converter_version = converter.version;
        let property_version = property.version;
        let file_name = AsciiPointer::empty();
        // AsciiPointer::from_cstr(cstr);
        Self {
            converter_version,
            property_version,
            file_name,
        }
    }
}

#[ffi_function]
#[no_mangle]
pub extern "C" fn read_default() -> Auth3d {
    const INPUT: &str = include_str!("../../assets/CAMPV001_BASE.a3da");
    let a3da = serde_divatree::de::from_str::<super::A3da>(INPUT).expect("deser error");
    a3da.into()
}

#[ffi_function]
#[no_mangle]
pub extern "C" fn read_from_string(s: AsciiPointer<'_>) -> Auth3d {
    let input = s.as_str();
    dbg!(s.as_c_str().map(|x| x.to_string_lossy()));
    if let Err(e) = &input {
        println!("Failed to convert string, {}", e);
    }
    let input = input.unwrap_or_default();
    let a3da = serde_divatree::de::from_str::<super::A3da>(input).map(Into::into);
    if let Err(e) = &a3da {
        println!("Failed to read a3da: {}", e);
    }
    a3da.unwrap_or_default()
}

// Guard function used by backends.
#[ffi_function]
#[no_mangle]
pub extern "C" fn api_guard() -> APIVersion {
    ffi_inventory().into()
}

fn ffi_inventory() -> Inventory {
    InventoryBuilder::new()
        .register(function!(api_guard))
        .register(function!(read_default))
        .register(function!(read_from_string))
        .inventory()
}

#[cfg(test)]
mod tests {
    use super::*;
    use interoptopus::Interop;

    #[test]
    fn bindings_cpython_cffi() {
        use interoptopus_backend_cpython::{Config, Generator};

        let library = ffi_inventory();

        Generator::new(Config::default(), library)
            .write_file("bindings/python/a3da.py")
            .unwrap();
    }

    #[test]
    fn bindings_c() {
        use interoptopus_backend_c::CIndentationStyle;
        use interoptopus_backend_c::CNamingStyle;
        use interoptopus_backend_c::{Config, Generator};

        Generator::new(
            Config {
                ifndef: "a3da".to_string(),
                prefix: "a3da_".to_string(),
                type_naming: CNamingStyle::UpperCamelCase,
                indentation: CIndentationStyle::KAndR,
                function_parameter_naming: CNamingStyle::LowerCamelCase,
                ..Config::default()
            },
            ffi_inventory(),
        )
        .write_file("bindings/c/a3da.h")
        .unwrap();
    }
}
