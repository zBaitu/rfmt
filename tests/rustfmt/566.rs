fn f() {
    let program = glium::Program::from_source(&display, &include_str!("./shaders/vertex.glsl"),
                                              &include_str!("./shaders/fragment.glsl"), None)
            .unwrap();
}
