use clap::{App, Arg};

pub fn parse_arg() -> clap::ArgMatches {
    return App::new("Preview Image Folder with auto refresh.")
    .version("0.1")
    .about("Show images in specified folder and refresh when images in folder is updated.")
    .arg(
        Arg::with_name("host")
            .long("host")
            .default_value("127.0.0.1")
            .value_name("HOST")
            .help("Set the host to listen for web page. If you use in container, you should set this into 0.0.0.0"),
    )
    .arg(
        Arg::with_name("port")
            .short('p')
            .long("port")
            .default_value("8000")
            .value_name("PORT")
            .help("Set the port to listen for web page."),
    )
    .arg(
        Arg::with_name("directory")
            .value_name("DIRECTORY")
            .default_value(".")
            .help("Directory which include images."), //.multiple(true),
    )
    .get_matches();
}
