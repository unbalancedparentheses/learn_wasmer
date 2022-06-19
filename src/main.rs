use wasmer::{imports, Global, Instance, Module, Store, Value, Function};

fn main() -> anyhow::Result<()> {
    let module_wat = r#"
    (module
        ;; reference to native factorial rust function. this enables wasm to call a rust function
        (func $factorial_native (import "env" "factorial_native") (param i32) (result i32))

        ;; global variable speed of light exported and set to 0 
        (global $speed_of_light (export "speed_of_light") (mut i32) (i32.const 0))
        ;; export a wasm function to rust that sets the speed of light global variable
        (func (export "set_speed_of_light") (param i32) (global.set $speed_of_light (local.get 0)))

        ;; definition of wasm multiply function 
        (func $multiply (param $lhs i32) (param $rhs i32) (result i32)
            get_local $lhs
            get_local $rhs
            i32.mul)
        ;; export the multiply function to rust
        (export "multiply" (func $multiply))

        ;; we now define a function factorial_add that calls the factorial_native rust function
        ;; with the first parameter and adds the second parameter
        (func $factorial_add (param $num i32) (param $add_number i32) (result i32)
            (call $factorial_native (local.get $num))
            get_local $add_number
            i32.add)
        ;; export factorial_add wasm function to rust
        (export "factorial_add" (func $factorial_add)))
    "#;

    let store = Store::default();
    let module = Module::new(&store, &module_wat)?;

    // create the factorial function that will be called from wasm
    fn factorial(num: i32) -> i32 {
        match num {
            0 | 1 => 1,
            _ => factorial(num - 1) * num,
        }
    }
    let factorial_native = Function::new_native(&store, factorial);

    // create an import object.
    let import_object = imports! {
        "env" => {
            "factorial_native" => factorial_native,
        }
    };

    let instance = Instance::new(&module, &import_object)?;

    let speed_of_light = instance.exports.get::<Global>("speed_of_light")?;
    println!("Speed of light value before set_speed_of_light: {:?}", speed_of_light.get());

    let set_speed_of_light = instance
        .exports
        .get_function("set_speed_of_light")?
        .native::<i32, ()>()?;
    set_speed_of_light.call(299792458)?;
    println!("Speed of light value after set_speed_of_light: {:?}", speed_of_light.get());
    assert_eq!(speed_of_light.get(), Value::I32(299792458));

    let multiply = instance.exports.get_function("multiply")?;
    let result = multiply.call(&[Value::I32(5), Value::I32(10)])?;
    assert_eq!(result[0], Value::I32(50));
    println!("Result : {:?}", result[0]);

    let multiply_native = multiply.native::<(i32, i32), i32>()?;
    let result_native = multiply_native.call(2, 2)?;
    assert_eq!(result_native, 4);
    println!("Result native: {:?}", result_native);

    // the wasm module exports a function called `factorial_add`. Let's get it.
    let factorial_add = instance
        .exports
        .get_function("factorial_add")?
        .native::<(i32, i32), i32>()?;

    println!("Calling `factorial_add` function...");
    // let's call the `factorial_add` exported function. It will call each
    // of the imported functions.
    let result = factorial_add.call(3, 2)?;

    println!("Results of `factorial_add`: {:?}", result);
    assert_eq!(result, 8);

    Ok(())
}
