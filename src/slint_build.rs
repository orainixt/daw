use slint_interpreter::{ComponentDefinition, Compiler, Value, SharedString, ComponentHandle};


struct SlintBuild{
}

impl SlintBuild {

    fn get_component(&self, code: String ,name: String) -> ComponentDefinition {
        let mut compiler = Compiler::default();

        // if future is async too, spin_on might not be enough
        let result = spin_on::spin_on(compiler.build_from_source(code.into(), Default::default())); 

        result.component(&name).unwrap()
    }

    fn build_playlist_bar(&self, name: String) {
        let code = r#"
            export component DynPlaylistBar {
                 
            } 
        "#;
    } 
}
