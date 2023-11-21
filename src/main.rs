extern crate wabt;
extern crate wasmi;

use core::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;
use wasmi::{ImportsBuilder, ModuleInstance, NopExternals};

fn main() {
    // Parse WAT (WebAssembly Text format) into wasm bytecode.
    let wasm_binary: Vec<u8> = wabt::wat2wasm(
        r#"
                (module
                (export "fib" (func $fib))
                (func $fib (param $n f32) (result f32)
                (local $result f32)
                (block
                (if
                    (f32.lt
                    (local.get $n)
                    (f32.const 2.0)
                    )
                    (then
                    (local.set $result (local.get $n))
                    (br 1)
                    )
                )
                (local.set $result
                    (f32.add
                    (call $fib
                    (f32.sub
                    (local.get $n)
                    (f32.const 2.0)
                    )
                    )
                    (call $fib
                    (f32.sub
                    (local.get $n)
                    (f32.const 1.0)
                    )
                    )
                    )
                )
                )
                (return (local.get $result))
                )
                )
            "#,
    )
    .expect("failed to parse wat");
    let tracer = wasmi::tracer::Tracer::new(HashMap::new(), &Vec::new());

    let module = wasmi::Module::from_buffer(&wasm_binary).expect("failed to load wasm");

    let tracer = Rc::new(RefCell::new(tracer));
    let instance = ModuleInstance::new(&module, &ImportsBuilder::default(), Some(tracer.clone()))
        .expect("failed to instantiate wasm module")
        .assert_no_start();

    assert_eq!(
        instance
            .invoke_export_trace(
                "fib",
                &[wasmi::RuntimeValue::F32(0x40c00000.into())],
                &mut NopExternals,
                tracer.clone(),
            )
            .expect("failed to execute"),
        Some(wasmi::RuntimeValue::F32(0x41000000.into()))
    );

    println!("{:?}", (*tracer).borrow());
}
