
use clap::{Args as clapArgs};
use tera::{Context, Tera};

#[derive(clapArgs)]
pub struct Args {
    /// Number of times to print
    #[clap(long="times", short='n')]
    iterations: u8,

    /// The Greeting message to use
    greeting: Option<String>,
}


pub fn template(args: &Args) {

    for i in 0 .. args.iterations {

        println!("Iteration {}", i);

        let mut context = Context::new();
        context.insert("greeting", &args.greeting);

        let rendered = Tera::one_off("{{ greeting }} world", &context, true);
        println!("Template.....");
        println!("{}", rendered.unwrap());
    }
}
