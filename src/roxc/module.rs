use cranelift::codegen;
use cranelift::prelude::*;
use cranelift_module::{default_libcall_names, Module};
use cranelift_object::{ObjectBackend, ObjectBuilder};
use cranelift_simplejit::{SimpleJITBackend, SimpleJITBuilder};
use target_lexicon::Triple;

pub(crate) fn init_object_module() -> Module<ObjectBackend> {
    let mut flags_builder = cranelift::codegen::settings::builder();
    flags_builder.enable("is_pic").unwrap();
    flags_builder.enable("enable_verifier").unwrap();
    let flags = settings::Flags::new(flags_builder);
    let isa = codegen::isa::lookup(Triple::host()).unwrap().finish(flags);

    // I'm not _totally_ sure what this second option does
    let builder = ObjectBuilder::new(isa, "roxc", default_libcall_names());
    cranelift_module::Module::new(builder)
}

pub(crate) fn init_simplejit_module() -> Module<SimpleJITBackend> {
    Module::new(SimpleJITBuilder::new(default_libcall_names()))
}
