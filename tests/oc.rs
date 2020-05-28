use objc::declare::ClassDecl;
use objc::runtime::{Object, Sel};
use objc::{class, msg_send, sel, sel_impl};
use objc_id::ShareId;

#[test]
pub fn test_objc() {
    let superclass = class!(NSObject);
    let mut decl = ClassDecl::new("MyNumber", superclass).unwrap();

    // Add an instance variable
    decl.add_ivar::<u32>("_number");

    // Add an ObjC method for getting the number
    extern "C" fn my_number_get(this: &Object, _cmd: Sel) -> u32 {
        unsafe { *this.get_ivar("_number") }
    }
    unsafe {
        decl.add_method(
            sel!(number),
            my_number_get as extern "C" fn(&Object, Sel) -> u32,
        );
    }

    decl.register();

    let cls = class!(MyNumber);
    let mut number: *mut Object = unsafe { msg_send![cls, new] };
    let number2 = unsafe { ShareId::from_ptr(number) };

    unsafe {
        (*number).set_ivar::<u32>("_number", 32);

        let n1 = (*number).get_ivar::<u32>("_number");
        println!("number {}", n1);

        let n2 = number2.get_ivar::<u32>("_number");
        println!("number2 {}", n2);
    }
}
