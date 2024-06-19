extern crate instrument;

use instrument::my_proc_macro;

#[allow(unused_imports)]

struct example{
    b : i32
}

#[my_proc_macro]
fn my_function() {
    let mut x:u8 = 10;
    static mut a: u8 = 2;
    let z = 4;
    let mut y:u8 = 5;
    let ptr = &mut x;
    println!("{:p}", ptr);
    let mut ss = example{b:2};
    //let mut my_struct = MyStruct { field: 0 };
    let mut my_array = [0; 5];

    start_atomic();
    y = 4;
    *ptr = 2;
    unsafe{a = a+ 2};
    //my_struct.field = 10;
    my_array[z -3] = 7;
    ss.b = 102;
    my_array[2] = my_array[3];
    end_atomic();

    //x = y + 3;
}
fn start_atomic() {
    // Placeholder for start logic
}

fn end_atomic() {
    // Placeholder for end logic
}

fn main() {
    my_function();
}

fn save_variables<T>(var_ptr: *const T, var_size: usize) {
    // Implementation of the function
    // For demonstration, let's print the address and size of the variable
    // In actual use, you might want to store this information in a log or use it differently
    println!("Variable at address: {:p}, size: {}", var_ptr, var_size);
}