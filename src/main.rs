fn main() {
    #[cfg(all(not(target_arch = "wasm32"), feature = "galaga"))]
    main::maverick_main();

    #[cfg(feature = "default")]
    println!("Enter valid game name with `cargo run --features game_name. Use lower case.`");
}

