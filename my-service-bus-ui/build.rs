fn main() {
    ci_utils::css::CssCompiler::new("./css")
        .add_file("01-tokens.css")
        .add_file("02-reset.css")
        .add_file("03-fonts.css")
        .add_file("04-layout.css")
        .add_file("05-sidebar.css")
        .add_file("06-topbar.css")
        .add_file("07-kpi.css")
        .add_file("08-primitives.css")
        .add_file("09-topic.css")
        .add_file("10-sessions.css")
        .add_file("11-dialog.css")
        .compile("./assets/app.css");
}
