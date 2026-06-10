use provide::{Provide, Request, provide_ref};

struct Foo {
    x: i32,
}

impl Provide for Foo {
    fn provide<'a>(&'a self, request: &mut Request<'a>) {
        request.provide_ref(&self.x);
    }
}

fn main() {
    let foo = Foo { x: 1 };
    let _x: Option<&'static i32> = provide_ref(&foo);
}
