use inkwell::context::Context;
use inkwell::module::Module;
use target_lexicon::Triple;

pub(crate) fn init_object_module() -> Module<'static> {
    let context = Context::create();
    context.create_module("rox") // TODO: Support multiple module names
}

pub(crate) fn init_simplejit_module() -> Module<'static> {
    init_object_module()
}
