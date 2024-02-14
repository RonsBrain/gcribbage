fn main() {
    glib_build_tools::compile_resources(
        &["data"],
        "data/resources.xml",
        "resources.gresource",
    )
}
