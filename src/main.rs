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
                (memory $0 1)
                (export "memory" (memory $0))
                (export "fibonacci" (func $fibonacci))
                (func $fibonacci (; 0 ;) (param $0 i32) (result i32)
                 (block $label$0
                  (br_if $label$0
                   (i32.ne
                    (i32.or
                     (local.get $0)
                     (i32.const 1)
                    )
                    (i32.const 1)
                   )
                  )
                  (return
                   (local.get $0)
                  )
                 )
                 (i32.add
                  (call $fibonacci
                   (i32.add
                    (local.get $0)
                    (i32.const -1)
                   )
                  )
                  (call $fibonacci
                   (i32.add
                    (local.get $0)
                    (i32.const -2)
                   )
                  )
                 )
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

    assert_eq!(instance.invoke_export_trace(
        "fibonacci",
        &[wasmi::RuntimeValue::I32(6)],
        &mut NopExternals,
        tracer.clone(),
    ).expect("failed to run export"), Some(wasmi::RuntimeValue::I32(8)));

    println!("{:?}", (*tracer).borrow());
}
