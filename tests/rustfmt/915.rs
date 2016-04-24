fn f() {
gfx_pipeline!(pipe {
        vbuf: gfx::VertexBuffer<Vertex> = (),
            out: gfx::RenderTarget<ColorFormat> = "Target0",
});
}
