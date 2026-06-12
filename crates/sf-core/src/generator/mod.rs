pub mod swift;
pub mod kotlin;
pub mod typescript;
pub mod go;

use crate::models::{StateMachine, Language};
use anyhow::Result;

pub trait CodeGenerator: Send + Sync {
    fn language(&self) -> Language;
    fn generate(&self, sm: &StateMachine) -> Result<String>;
    fn file_extension(&self) -> &str;
}

pub fn generate(sm: &StateMachine, lang: &Language) -> Result<String> {
    let gen: Box<dyn CodeGenerator> = match lang {
        Language::Swift      => Box::new(swift::SwiftGenerator),
        Language::Kotlin     => Box::new(kotlin::KotlinGenerator),
        Language::TypeScript => Box::new(typescript::TypeScriptGenerator),
        Language::Go         => Box::new(go::GoGenerator),
        _                    => Box::new(typescript::TypeScriptGenerator),
    };
    gen.generate(sm)
}
