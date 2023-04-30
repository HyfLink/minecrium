use bevy::prelude::*;

fn main() {
    App::new()
        .add_system(hello_system)
        .add_system(world_system)
        .run()
}

fn hello_system() {
    println!("Hello");
}

fn world_system() {
    println!("World");
}
